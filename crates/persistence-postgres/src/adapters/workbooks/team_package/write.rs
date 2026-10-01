use super::names::{ensure_team_name_alias, preserve_current_team_canonical_alias};
use super::values::{
    clear_fields, common_metadata, normalize_name, normalize_team_type, object, optional_bool,
    optional_date, optional_datetime, optional_f64, optional_i16, optional_uuid,
    parse_bool_default, parse_date, parse_f64_default, parse_i32_default, require_text,
    resolved_uuid, source_urls, text,
};
use super::CommitOutcome;
use crate::{PersistenceError, PersistenceResult};
use chrono::Utc;
use football_domain::{
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportRow, SpreadsheetRowStatus,
};
use serde_json::{Map, Value};
use sqlx::{Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

pub(super) async fn execute_team_monthly_row(
    tx: &mut Transaction<'_, Postgres>,
    row: &SpreadsheetImportRow,
) -> PersistenceResult<CommitOutcome> {
    let values = object(&row.payload)?;
    let metadata = common_metadata(values);
    match row.entity_type {
        SpreadsheetEntityType::Team => {
            if row.status == SpreadsheetRowStatus::ReadyAdd {
                let id = optional_uuid(values, "team_id")?.unwrap_or_else(Uuid::new_v4);
                let name = require_text(values, "official_name")?;
                sqlx::query("INSERT INTO football.teams (id,canonical_name,normalized_name,country_code,is_active,metadata) VALUES ($1,$2,$3,$4,$5,$6)")
                    .bind(id).bind(&name).bind(normalize_name(&name)).bind(text(values,"country_code"))
                    .bind(parse_bool_default(text(values,"is_active").as_deref(), true)?).bind(&metadata)
                    .execute(&mut **tx).await?;
                upsert_team_profile_from_values(tx, id, values, &metadata).await?;
                ensure_team_name_alias(tx, id, &name, None, None, None, &metadata).await?;
                if let Some(short_name) = text(values, "short_name") {
                    ensure_team_name_alias(tx, id, &short_name, None, None, None, &metadata)
                        .await?;
                }
                Ok(CommitOutcome {
                    inserted: 1,
                    ..Default::default()
                })
            } else {
                let id = resolved_uuid(values, "team")?;
                apply_team_update(tx, id, row.action, values, &metadata).await?;
                Ok(CommitOutcome {
                    updated: 1,
                    ..Default::default()
                })
            }
        }
        SpreadsheetEntityType::TeamName => {
            let team_id = resolve_entity_id_tx(tx, values, "team").await?;
            let name = require_text(values, "name_value")?;
            let normalized_name = normalize_name(&name);
            let is_primary = parse_bool_default(text(values, "is_primary").as_deref(), false)?;
            if is_primary {
                preserve_current_team_canonical_alias(tx, team_id, &metadata).await?;
                sqlx::query(
                    "UPDATE football.teams SET canonical_name=$2, normalized_name=$3, updated_at=now() WHERE id=$1",
                )
                .bind(team_id)
                .bind(&name)
                .bind(&normalized_name)
                .execute(&mut **tx)
                .await?;
            }
            let language_code = text(values, "language_code");
            let valid_from = optional_date(values, "valid_from")?;
            let valid_to = optional_date(values, "valid_to")?;
            ensure_team_name_alias(
                tx,
                team_id,
                &name,
                language_code.as_deref(),
                valid_from,
                valid_to,
                &metadata,
            )
            .await?;
            Ok(CommitOutcome {
                inserted: 1,
                ..Default::default()
            })
        }
        SpreadsheetEntityType::Coach => {
            if row.status == SpreadsheetRowStatus::ReadyAdd {
                let id = optional_uuid(values, "coach_id")?.unwrap_or_else(Uuid::new_v4);
                let name = require_text(values, "official_name")?;
                sqlx::query("INSERT INTO football.coaches (id,canonical_name,normalized_name,nationality_code,status,metadata) VALUES ($1,$2,$3,$4,$5,$6)")
                    .bind(id).bind(&name).bind(normalize_name(&name)).bind(text(values,"nationality_code"))
                    .bind(text(values,"coach_status").unwrap_or_else(|| "active".into())).bind(metadata)
                    .execute(&mut **tx).await?;
                Ok(CommitOutcome {
                    inserted: 1,
                    ..Default::default()
                })
            } else {
                let id = resolved_uuid(values, "coach")?;
                let clear = clear_fields(values);
                sqlx::query(r#"UPDATE football.coaches SET
                    canonical_name=COALESCE(NULLIF($2,''),canonical_name),
                    normalized_name=CASE WHEN NULLIF($2,'') IS NULL THEN normalized_name ELSE $3 END,
                    nationality_code=CASE WHEN $4 THEN NULL ELSE COALESCE(NULLIF($5,''),nationality_code) END,
                    status=COALESCE(NULLIF($6,''),status), metadata=metadata || $7, updated_at=now() WHERE id=$1"#)
                    .bind(id).bind(text(values,"official_name").unwrap_or_default()).bind(normalize_name(&text(values,"official_name").unwrap_or_default()))
                    .bind(clear.contains("nationality_code")).bind(text(values,"nationality_code"))
                    .bind(text(values,"coach_status").unwrap_or_default()).bind(metadata)
                    .execute(&mut **tx).await?;
                Ok(CommitOutcome {
                    updated: 1,
                    ..Default::default()
                })
            }
        }
        SpreadsheetEntityType::TeamCoachPeriod => {
            let team_id = resolve_entity_id_tx(tx, values, "team").await?;
            let coach_id = resolve_entity_id_tx(tx, values, "coach").await?;
            let role = require_text(values, "role")?;
            let valid_from = parse_date(require_text(values, "valid_from")?, "valid_from")?;
            let valid_to = optional_date(values, "valid_to")?;
            let mut ended = 0;
            if valid_to.is_none()
                && matches!(
                    role.as_str(),
                    "head_coach" | "interim_head_coach" | "caretaker"
                )
            {
                ended = sqlx::query(r#"UPDATE football.team_coach_periods SET valid_to=$2 - 1
                    WHERE team_id=$1 AND valid_to IS NULL AND valid_from < $2
                      AND role IN ('head_coach','interim_head_coach','caretaker') AND coach_id<>$3"#)
                    .bind(team_id).bind(valid_from).bind(coach_id).execute(&mut **tx).await?.rows_affected();
            }
            sqlx::query(r#"INSERT INTO football.team_coach_periods
                (id,team_id,coach_id,role,valid_from,valid_to,is_interim,confidence,metadata)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                ON CONFLICT (team_id,coach_id,role,valid_from) DO UPDATE SET
                    valid_to=EXCLUDED.valid_to,is_interim=EXCLUDED.is_interim,
                    confidence=EXCLUDED.confidence,metadata=football.team_coach_periods.metadata || EXCLUDED.metadata"#)
                .bind(Uuid::new_v4()).bind(team_id).bind(coach_id).bind(role).bind(valid_from).bind(valid_to)
                .bind(parse_bool_default(text(values,"is_interim").as_deref(), false)?)
                .bind(parse_f64_default(text(values,"confidence").as_deref(),0.5,"confidence")?).bind(metadata)
                .execute(&mut **tx).await?;
            Ok(CommitOutcome {
                inserted: 1,
                ended_previous: ended,
                ..Default::default()
            })
        }
        SpreadsheetEntityType::TeamTacticalObservation => {
            let team_id = resolve_entity_id_tx(tx, values, "team").await?;
            let coach_id = resolve_optional_entity_id_tx(tx, values, "coach").await?;
            sqlx::query(r#"INSERT INTO feature.team_tactical_observations (
                id,team_id,coach_id,window_start,window_end,build_up_style,progression_style,
                attacking_width,pressing_intensity,defensive_block,transition_speed,set_piece_tendency,
                tactical_summary,confidence,source_urls,verified_at,observed_at,metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
            ON CONFLICT DO NOTHING"#)
                .bind(Uuid::new_v4()).bind(team_id).bind(coach_id)
                .bind(parse_date(require_text(values,"window_start")?,"window_start")?)
                .bind(parse_date(require_text(values,"window_end")?,"window_end")?)
                .bind(text(values,"build_up_style")).bind(text(values,"progression_style"))
                .bind(text(values,"attacking_width")).bind(text(values,"pressing_intensity"))
                .bind(text(values,"defensive_block")).bind(text(values,"transition_speed"))
                .bind(text(values,"set_piece_tendency")).bind(text(values,"tactical_summary"))
                .bind(parse_f64_default(text(values,"confidence").as_deref(),0.5,"confidence")?)
                .bind(source_urls(values)).bind(optional_datetime(values,"verified_at")?)
                .bind(optional_datetime(values,"observed_at")?.unwrap_or_else(Utc::now)).bind(metadata)
                .execute(&mut **tx).await?;
            Ok(CommitOutcome {
                inserted: 1,
                ..Default::default()
            })
        }
        SpreadsheetEntityType::TeamAbilityObservation => {
            let team_id = resolve_entity_id_tx(tx, values, "team").await?;
            sqlx::query(r#"INSERT INTO feature.team_ability_observations (
                id,team_id,observed_at,window_start,window_end,attack_rating,midfield_rating,
                defence_rating,goalkeeper_rating,squad_depth_rating,stability_rating,sample_size,
                methodology,confidence,source_urls,verified_at,metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            ON CONFLICT (team_id,observed_at,window_start,window_end) DO UPDATE SET
                attack_rating=EXCLUDED.attack_rating,midfield_rating=EXCLUDED.midfield_rating,
                defence_rating=EXCLUDED.defence_rating,goalkeeper_rating=EXCLUDED.goalkeeper_rating,
                squad_depth_rating=EXCLUDED.squad_depth_rating,stability_rating=EXCLUDED.stability_rating,
                sample_size=EXCLUDED.sample_size,methodology=EXCLUDED.methodology,confidence=EXCLUDED.confidence,
                source_urls=EXCLUDED.source_urls,verified_at=EXCLUDED.verified_at,
                metadata=feature.team_ability_observations.metadata || EXCLUDED.metadata"#)
                .bind(Uuid::new_v4()).bind(team_id)
                .bind(optional_datetime(values,"observed_at")?.unwrap_or_else(Utc::now))
                .bind(parse_date(require_text(values,"window_start")?,"window_start")?)
                .bind(parse_date(require_text(values,"window_end")?,"window_end")?)
                .bind(optional_f64(values,"attack_rating")?).bind(optional_f64(values,"midfield_rating")?)
                .bind(optional_f64(values,"defence_rating")?).bind(optional_f64(values,"goalkeeper_rating")?)
                .bind(optional_f64(values,"squad_depth_rating")?).bind(optional_f64(values,"stability_rating")?)
                .bind(parse_i32_default(text(values,"sample_size").as_deref(),0,"sample_size")?)
                .bind(text(values,"methodology"))
                .bind(parse_f64_default(text(values,"confidence").as_deref(),0.5,"confidence")?)
                .bind(source_urls(values)).bind(optional_datetime(values,"verified_at")?).bind(metadata)
                .execute(&mut **tx).await?;
            Ok(CommitOutcome {
                inserted: 1,
                ..Default::default()
            })
        }
        _ => Err(PersistenceError::InvalidState(
            "球队月度工作簿包含不支持的实体".into(),
        )),
    }
}

async fn upsert_team_profile_from_values(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    values: &Map<String, Value>,
    metadata: &Value,
) -> PersistenceResult<()> {
    let team_type =
        normalize_team_type(text(values, "team_type"))?.unwrap_or_else(|| "club".to_string());
    sqlx::query(r#"INSERT INTO football.team_profiles (
        team_id,short_name,team_type,founded_year,city,stadium,data_confidence,notes,metadata,updated_at
    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,now())
    ON CONFLICT (team_id) DO UPDATE SET
        short_name=COALESCE(EXCLUDED.short_name,football.team_profiles.short_name),
        team_type=COALESCE(NULLIF(EXCLUDED.team_type,''),football.team_profiles.team_type),
        founded_year=COALESCE(EXCLUDED.founded_year,football.team_profiles.founded_year),
        city=COALESCE(EXCLUDED.city,football.team_profiles.city),stadium=COALESCE(EXCLUDED.stadium,football.team_profiles.stadium),
        data_confidence=EXCLUDED.data_confidence,notes=COALESCE(EXCLUDED.notes,football.team_profiles.notes),
        metadata=football.team_profiles.metadata || EXCLUDED.metadata,updated_at=now()"#)
        .bind(team_id).bind(text(values,"short_name")).bind(team_type)
        .bind(optional_i16(values,"founded_year")?).bind(text(values,"city")).bind(text(values,"stadium"))
        .bind(parse_f64_default(text(values,"data_confidence").as_deref(),0.5,"data_confidence")?)
        .bind(text(values,"notes")).bind(metadata).execute(&mut **tx).await?;
    Ok(())
}

async fn apply_team_update(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    action: SpreadsheetAction,
    values: &Map<String, Value>,
    metadata: &Value,
) -> PersistenceResult<()> {
    let clear = if action == SpreadsheetAction::Clear {
        clear_fields(values)
    } else {
        HashSet::new()
    };
    let name = text(values, "official_name").unwrap_or_default();
    let team_type = normalize_team_type(text(values, "team_type"))?.unwrap_or_default();
    if !name.trim().is_empty() {
        preserve_current_team_canonical_alias(tx, team_id, metadata).await?;
    }
    sqlx::query(
        r#"UPDATE football.teams SET
        canonical_name=COALESCE(NULLIF($2,''),canonical_name),
        normalized_name=CASE WHEN NULLIF($2,'') IS NULL THEN normalized_name ELSE $3 END,
        country_code=CASE WHEN $4 THEN NULL ELSE COALESCE(NULLIF($5,''),country_code) END,
        is_active=COALESCE($6,is_active), metadata=metadata || $7, updated_at=now() WHERE id=$1"#,
    )
    .bind(team_id)
    .bind(&name)
    .bind(normalize_name(&name))
    .bind(clear.contains("country_code"))
    .bind(text(values, "country_code"))
    .bind(optional_bool(values, "is_active")?)
    .bind(metadata)
    .execute(&mut **tx)
    .await?;
    if !name.trim().is_empty() {
        ensure_team_name_alias(tx, team_id, &name, None, None, None, metadata).await?;
    }
    if let Some(short_name) = text(values, "short_name") {
        ensure_team_name_alias(tx, team_id, &short_name, None, None, None, metadata).await?;
    }
    sqlx::query(
        r#"INSERT INTO football.team_profiles (team_id, team_type, data_confidence, metadata, updated_at)
        VALUES ($1, 'club', 0.5, $2, now())
        ON CONFLICT (team_id) DO UPDATE SET
            short_name=CASE WHEN $3 THEN NULL ELSE COALESCE(NULLIF($4,''),football.team_profiles.short_name) END,
            team_type=COALESCE(NULLIF($5,''),football.team_profiles.team_type),
            founded_year=CASE WHEN $6 THEN NULL ELSE COALESCE($7,football.team_profiles.founded_year) END,
            city=CASE WHEN $8 THEN NULL ELSE COALESCE(NULLIF($9,''),football.team_profiles.city) END,
            stadium=CASE WHEN $10 THEN NULL ELSE COALESCE(NULLIF($11,''),football.team_profiles.stadium) END,
            data_confidence=COALESCE($12,football.team_profiles.data_confidence),
            notes=CASE WHEN $13 THEN NULL ELSE COALESCE(NULLIF($14,''),football.team_profiles.notes) END,
            metadata=football.team_profiles.metadata || EXCLUDED.metadata,
            updated_at=now()"#,
    )
    .bind(team_id)
    .bind(metadata)
    .bind(clear.contains("short_name"))
    .bind(text(values, "short_name").unwrap_or_default())
    .bind(team_type)
    .bind(clear.contains("founded_year"))
    .bind(optional_i16(values, "founded_year")?)
    .bind(clear.contains("city"))
    .bind(text(values, "city").unwrap_or_default())
    .bind(clear.contains("stadium"))
    .bind(text(values, "stadium").unwrap_or_default())
    .bind(optional_f64(values, "data_confidence")?)
    .bind(clear.contains("notes"))
    .bind(text(values, "notes").unwrap_or_default())
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn resolve_entity_id_tx(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    prefix: &str,
) -> PersistenceResult<Uuid> {
    if let Some(id) = optional_uuid(values, &format!("_resolved_{prefix}_id"))?
        .or(optional_uuid(values, &format!("{prefix}_id"))?)
    {
        return Ok(id);
    }
    let name = text(values, &format!("{prefix}_name"))
        .or_else(|| text(values, "official_name"))
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少 {prefix} 名称")))?;
    let table = if prefix == "team" {
        "football.teams"
    } else {
        "football.coaches"
    };
    let ids = sqlx::query_scalar::<_, Uuid>(&format!(
        "SELECT id FROM {table} WHERE normalized_name=$1 ORDER BY id LIMIT 2"
    ))
    .bind(normalize_name(&name))
    .fetch_all(&mut **tx)
    .await?;
    match ids.as_slice() {
        [id] => Ok(*id),
        [] => Err(PersistenceError::InvalidState(format!(
            "提交时无法匹配 {prefix}: {name}"
        ))),
        _ => Err(PersistenceError::InvalidState(format!(
            "提交时 {prefix} 仍存在多个候选: {name}"
        ))),
    }
}

async fn resolve_optional_entity_id_tx(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    prefix: &str,
) -> PersistenceResult<Option<Uuid>> {
    let has = optional_uuid(values, &format!("_resolved_{prefix}_id"))?.is_some()
        || optional_uuid(values, &format!("{prefix}_id"))?.is_some()
        || text(values, &format!("{prefix}_name")).is_some();
    if has {
        Ok(Some(resolve_entity_id_tx(tx, values, prefix).await?))
    } else {
        Ok(None)
    }
}
