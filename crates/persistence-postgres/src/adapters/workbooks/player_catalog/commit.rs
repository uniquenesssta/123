use super::values::{
    default_text, normalize_name, optional_bool, optional_date, optional_datetime, optional_f64,
    optional_i16, optional_i32, optional_text, optional_uuid, payload_optional_uuid, payload_uuid,
    required_date, required_datetime, required_f64, required_text, spreadsheet_clear_fields,
    spreadsheet_row_metadata, text,
};
use crate::adapters::catalog::references::write_external_entity_id;
use crate::adapters::workbooks::batch_ledger::{
    batch::{self as ledger, ImportFamily},
    mapping::{
        player_parse_action as parse_action, player_parse_entity_type as parse_entity_type,
        player_parse_row_status as parse_row_status,
    },
    rows as ledger_rows,
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{
    ExternalEntityIdDraft, SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportCommitResult,
    SpreadsheetRowStatus,
};
use serde_json::{json, Map, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

impl PostgresStore {
    pub async fn commit_spreadsheet_import(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportCommitResult> {
        let mut tx = self.pool.begin().await?;
        let batch = ledger::lock_batch_in_tx(&mut tx, batch_id, ImportFamily::Player).await?;
        let status: String = batch.try_get("status")?;
        if status == "succeeded" {
            return Ok(SpreadsheetImportCommitResult {
                batch_id,
                inserted_count: batch.try_get::<i64, _>("inserted_count")? as u64,
                updated_count: batch.try_get::<i64, _>("updated_count")? as u64,
                ended_previous_count: batch.try_get::<i64, _>("ended_previous_count")? as u64,
                skipped_count: batch.try_get::<i64, _>("skipped_count")? as u64,
                error_count: batch.try_get::<i64, _>("error_count")? as u64,
                finished_at: batch
                    .try_get::<Option<DateTime<Utc>>, _>("finished_at")?
                    .unwrap_or_else(Utc::now),
            });
        }
        ledger::require_pending(&status, format!("导入批次状态为 {status}，不能提交"))?;
        let blocking_count: i64 = ledger_rows::blocking_rows_in_tx(&mut tx, batch_id).await?;
        if blocking_count > 0 {
            return Err(PersistenceError::InvalidState(format!(
                "仍有 {blocking_count} 条冲突或错误记录，不能提交"
            )));
        }
        ledger::start_batch_in_tx(&mut tx, batch_id).await?;
        let rows = ledger_rows::commit_rows_in_tx(&mut tx, batch_id, ImportFamily::Player).await?;
        let mut player_keys = HashMap::<String, Uuid>::new();
        let mut team_keys = HashMap::<String, Uuid>::new();
        let mut inserted = 0_u64;
        let mut updated = 0_u64;
        let mut context = ImportCommitContext {
            player_keys: &mut player_keys,
            team_keys: &mut team_keys,
        };
        for row in rows {
            let row_id: Uuid = row.try_get("id")?;
            let entity_type = parse_entity_type(&row.try_get::<String, _>("entity_type")?)?;
            let action = parse_action(&row.try_get::<String, _>("requested_action")?)?;
            let row_status = parse_row_status(&row.try_get::<String, _>("status")?)?;
            let payload: Value = row.try_get("payload")?;
            let matched_entity_id: Option<Uuid> = row.try_get("matched_entity_id")?;
            let outcome = apply_import_row(
                &mut tx,
                entity_type,
                action,
                row_status,
                &payload,
                matched_entity_id,
                &mut context,
            )
            .await?;
            match outcome {
                ApplyOutcome::Inserted => inserted += 1,
                ApplyOutcome::Updated => updated += 1,
            }
            ledger_rows::mark_imported_in_tx(&mut tx, row_id).await?;
        }
        let skipped: i64 = ledger_rows::skipped_rows_in_tx(&mut tx, batch_id).await?;
        let finished_at = Utc::now();
        let result = SpreadsheetImportCommitResult {
            batch_id,
            inserted_count: inserted,
            updated_count: updated,
            ended_previous_count: 0,
            skipped_count: skipped as u64,
            error_count: 0,
            finished_at,
        };
        ledger::finish_batch_in_tx(&mut tx, &result, ImportFamily::Player).await?;
        tx.commit().await?;
        Ok(result)
    }
}
async fn apply_import_row(
    tx: &mut Transaction<'_, Postgres>,
    entity_type: SpreadsheetEntityType,
    action: SpreadsheetAction,
    status: SpreadsheetRowStatus,
    payload: &Value,
    matched_entity_id: Option<Uuid>,
    context: &mut ImportCommitContext<'_>,
) -> PersistenceResult<ApplyOutcome> {
    let values = payload
        .as_object()
        .ok_or_else(|| PersistenceError::InvalidState("导入 payload 无效".to_string()))?;
    match entity_type {
        SpreadsheetEntityType::TeamName
        | SpreadsheetEntityType::Coach
        | SpreadsheetEntityType::CoachName
        | SpreadsheetEntityType::TeamCoachPeriod
        | SpreadsheetEntityType::FormationUsage
        | SpreadsheetEntityType::TeamTacticalObservation
        | SpreadsheetEntityType::TeamAbilityObservation => Err(PersistenceError::InvalidState(
            "球队月度实体必须使用球队月度导入流程".to_string(),
        )),
        SpreadsheetEntityType::Match
        | SpreadsheetEntityType::Lineup
        | SpreadsheetEntityType::LineupPlayer => Err(PersistenceError::InvalidState(
            "比赛与阵容必须使用专用导入流程".to_string(),
        )),
        SpreadsheetEntityType::Team => {
            let id = match matched_entity_id {
                Some(id) => id,
                None => optional_uuid(values, "team_id")?.unwrap_or_else(Uuid::new_v4),
            };
            let metadata = spreadsheet_row_metadata(values);
            let outcome = if matched_entity_id.is_some() {
                let name = text(values, "official_name");
                let clear = if action == SpreadsheetAction::Clear {
                    spreadsheet_clear_fields(values)
                } else {
                    HashSet::new()
                };
                sqlx::query(
                    r#"UPDATE football.teams SET
                    canonical_name=COALESCE(NULLIF($2,''),canonical_name),
                    normalized_name=CASE WHEN NULLIF($2,'') IS NULL THEN normalized_name ELSE $3 END,
                    country_code=CASE WHEN $4 THEN NULL ELSE COALESCE(NULLIF($5,''),country_code) END,
                    is_active=COALESCE($6,is_active), metadata=metadata || $7, updated_at=now()
                    WHERE id=$1"#,
                )
                .bind(id)
                .bind(&name)
                .bind(normalize_name(&name))
                .bind(clear.contains("country_code"))
                .bind(optional_text(values, "country_code"))
                .bind(optional_bool(values, "is_active")?)
                .bind(&metadata)
                .execute(&mut **tx)
                .await?;
                ApplyOutcome::Updated
            } else {
                let name = required_text(values, "official_name")?;
                sqlx::query("INSERT INTO football.teams (id, canonical_name, normalized_name, country_code, is_active, metadata) VALUES ($1,$2,$3,$4,$5,$6)")
                    .bind(id).bind(&name).bind(normalize_name(&name))
                    .bind(optional_text(values, "country_code"))
                    .bind(optional_bool(values, "is_active")?.unwrap_or(true))
                    .bind(&metadata).execute(&mut **tx).await?;
                ApplyOutcome::Inserted
            };
            let key = text(values, "team_key");
            if !key.is_empty() {
                context.team_keys.insert(key, id);
            }
            Ok(outcome)
        }
        SpreadsheetEntityType::Player => {
            let id = match matched_entity_id {
                Some(id) => id,
                None => optional_uuid(values, "player_id")?.unwrap_or_else(Uuid::new_v4),
            };
            let metadata = spreadsheet_row_metadata(values);
            let outcome = if matched_entity_id.is_some() {
                let name = text(values, "official_name");
                let normalized_name = normalize_name(&name);
                let clear = if action == SpreadsheetAction::Clear {
                    spreadsheet_clear_fields(values)
                } else {
                    HashSet::new()
                };
                let previous_normalized_name: String = sqlx::query_scalar(
                    "SELECT normalized_name FROM football.players WHERE id = $1 FOR UPDATE",
                )
                .bind(id)
                .fetch_one(&mut **tx)
                .await?;
                sqlx::query(
                    r#"UPDATE football.players SET
                    canonical_name=COALESCE(NULLIF($2,''),canonical_name),
                    normalized_name=CASE WHEN NULLIF($2,'') IS NULL THEN normalized_name ELSE $3 END,
                    date_of_birth=CASE WHEN $4 THEN NULL ELSE COALESCE($5,date_of_birth) END,
                    nationality_code=CASE WHEN $6 THEN NULL ELSE COALESCE(NULLIF($7,''),nationality_code) END,
                    preferred_foot=CASE WHEN $8 THEN 'unknown' ELSE COALESCE(NULLIF($9,''),preferred_foot) END,
                    height_cm=CASE WHEN $10 THEN NULL ELSE COALESCE($11,height_cm) END,
                    status=CASE WHEN $12 THEN 'unknown' ELSE COALESCE(NULLIF($13,''),status) END,
                    metadata=metadata || $14, updated_at=now() WHERE id=$1"#,
                )
                .bind(id)
                .bind(&name)
                .bind(&normalized_name)
                .bind(clear.contains("birth_date"))
                .bind(optional_date(values, "birth_date")?)
                .bind(clear.contains("nationality_code"))
                .bind(optional_text(values, "nationality_code"))
                .bind(clear.contains("preferred_foot"))
                .bind(optional_text(values, "preferred_foot"))
                .bind(clear.contains("height_cm"))
                .bind(optional_i16(values, "height_cm")?)
                .bind(clear.contains("player_status"))
                .bind(optional_text(values, "player_status"))
                .bind(&metadata)
                .execute(&mut **tx)
                .await?;
                if !name.is_empty() && previous_normalized_name != normalized_name {
                    sqlx::query(
                        "UPDATE football.player_names SET is_primary=false WHERE player_id=$1",
                    )
                    .bind(id)
                    .execute(&mut **tx)
                    .await?;
                    sqlx::query("INSERT INTO football.player_names (id,player_id,name,normalized_name,is_primary,metadata) VALUES ($1,$2,$3,$4,true,$5)")
                        .bind(Uuid::new_v4()).bind(id).bind(&name).bind(&normalized_name)
                        .bind(&metadata).execute(&mut **tx).await?;
                }
                ApplyOutcome::Updated
            } else {
                let name = required_text(values, "official_name")?;
                let birth = optional_date(values, "birth_date")?;
                let nationality = optional_text(values, "nationality_code");
                let foot = default_text(values, "preferred_foot", "unknown");
                let height = optional_i16(values, "height_cm")?;
                let player_status = default_text(values, "player_status", "active");
                let normalized_name = normalize_name(&name);
                sqlx::query("INSERT INTO football.players (id, canonical_name, normalized_name, date_of_birth, nationality_code, preferred_foot, height_cm, status, metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                    .bind(id).bind(&name).bind(&normalized_name).bind(birth).bind(nationality)
                    .bind(&foot).bind(height).bind(&player_status).bind(&metadata)
                    .execute(&mut **tx).await?;
                sqlx::query("INSERT INTO football.player_names (id, player_id, name, normalized_name, is_primary, metadata) VALUES ($1,$2,$3,$4,true,$5)")
                    .bind(Uuid::new_v4()).bind(id).bind(&name).bind(&normalized_name)
                    .bind(&metadata).execute(&mut **tx).await?;
                ApplyOutcome::Inserted
            };
            let key = text(values, "player_key");
            if !key.is_empty() {
                context.player_keys.insert(key, id);
            }
            Ok(outcome)
        }
        SpreadsheetEntityType::PlayerName => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            let name = required_text(values, "name_value")?;
            let is_primary = optional_bool(values, "is_primary")?.unwrap_or(false);
            if is_primary {
                sqlx::query("UPDATE football.player_names SET is_primary=false WHERE player_id=$1")
                    .bind(player_id)
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("UPDATE football.players SET canonical_name=$2, normalized_name=$3, updated_at=now() WHERE id=$1").bind(player_id).bind(&name).bind(normalize_name(&name)).execute(&mut **tx).await?;
            }
            sqlx::query("INSERT INTO football.player_names (id,player_id,name,normalized_name,language_code,is_primary,valid_from,valid_to,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                .bind(Uuid::new_v4()).bind(player_id).bind(&name).bind(normalize_name(&name)).bind(optional_text(values,"language_code"))
                .bind(is_primary).bind(optional_date(values,"valid_from")?).bind(optional_date(values,"valid_to")?)
                .bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerPosition => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            let code = required_text(values, "position_code")?.to_uppercase();
            let is_primary = optional_bool(values, "is_primary")?.unwrap_or(false);
            if is_primary {
                sqlx::query(
                    "UPDATE football.player_positions SET is_primary=false WHERE player_id=$1",
                )
                .bind(player_id)
                .execute(&mut **tx)
                .await?;
            }
            let default_role_code = optional_text(values, "default_role_code")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let clear_default_role = action == SpreadsheetAction::Clear
                && spreadsheet_clear_fields(values).contains("default_role_code");
            sqlx::query("INSERT INTO football.player_positions (id,player_id,position_code,proficiency,default_role_code,is_primary,valid_from,valid_to,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (player_id, position_code, valid_from) DO UPDATE SET proficiency=EXCLUDED.proficiency,default_role_code=CASE WHEN $10 THEN NULL WHEN EXCLUDED.default_role_code IS NULL THEN football.player_positions.default_role_code ELSE EXCLUDED.default_role_code END,is_primary=EXCLUDED.is_primary,valid_to=EXCLUDED.valid_to,metadata=football.player_positions.metadata||EXCLUDED.metadata")
                .bind(Uuid::new_v4()).bind(player_id).bind(code).bind(required_f64(values,"proficiency")?).bind(default_role_code).bind(is_primary)
                .bind(optional_date(values,"valid_from")?).bind(optional_date(values,"valid_to")?)
                .bind(spreadsheet_row_metadata(values)).bind(clear_default_role).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerTeamPeriod => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            let team_id = if values.get("_auto_create_team").and_then(Value::as_bool) == Some(true)
            {
                resolve_or_create_import_team(tx, values).await?
            } else {
                resolve_committed_team_id(tx, values, context).await?
            };
            sqlx::query("INSERT INTO football.player_team_periods (id,player_id,team_id,season_id,squad_number,valid_from,valid_to,registration_status,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                .bind(Uuid::new_v4()).bind(player_id).bind(team_id).bind(optional_uuid(values,"season_id")?).bind(optional_i16(values,"squad_number")?)
                .bind(required_date(values,"valid_from")?).bind(optional_date(values,"valid_to")?).bind(default_text(values,"registration_status","registered"))
                .bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerAbility => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            sqlx::query("INSERT INTO feature.player_ability_observations (id,player_id,dimension_code,context_type,context_id,value,confidence,sample_size,observed_at,effective_from,effective_to,calculation_version,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)")
                .bind(Uuid::new_v4()).bind(player_id).bind(required_text(values,"dimension_code")?).bind(default_text(values,"context_type","general"))
                .bind(optional_uuid(values,"context_id")?).bind(required_f64(values,"value")?).bind(optional_f64(values,"confidence")?.unwrap_or(1.0))
                .bind(optional_i32(values,"sample_size")?.unwrap_or(1)).bind(required_datetime(values,"observed_at")?).bind(required_datetime(values,"effective_from")?)
                .bind(optional_datetime(values,"effective_to")?).bind(required_text(values,"calculation_version")?).bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerAvailability => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            let team_id = resolve_optional_committed_team_id(tx, values, context).await?;
            sqlx::query("INSERT INTO football.player_availability (id,player_id,team_id,competition_id,status,reason,confidence,valid_from,valid_to,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
                .bind(Uuid::new_v4()).bind(player_id).bind(team_id).bind(optional_uuid(values,"competition_id")?).bind(required_text(values,"availability_status")?)
                .bind(optional_text(values,"reason")).bind(optional_f64(values,"confidence")?.unwrap_or(1.0)).bind(required_datetime(values,"valid_from")?)
                .bind(optional_datetime(values,"valid_to")?).bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            sqlx::query(
                r#"
                INSERT INTO feature.player_dynamic_tags (
                    id, player_id, tag_code, value, label, confidence,
                    observed_at, valid_from, valid_to, competition_id,
                    position_code, opponent_team_id, sample_size, source_type,
                    calculation_version, metadata
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10,
                    $11, $12, $13, $14,
                    $15, $16
                )
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(player_id)
            .bind(required_text(values, "tag_code")?)
            .bind(required_f64(values, "tag_value")?)
            .bind(optional_text(values, "label"))
            .bind(optional_f64(values, "confidence")?.unwrap_or(1.0))
            .bind(required_datetime(values, "observed_at")?)
            .bind(required_datetime(values, "valid_from")?)
            .bind(required_datetime(values, "valid_to")?)
            .bind(optional_uuid(values, "competition_id")?)
            .bind(optional_text(values, "position_code").map(|value| value.to_uppercase()))
            .bind(optional_uuid(values, "opponent_team_id")?)
            .bind(optional_i32(values, "sample_size")?.unwrap_or(1))
            .bind(default_text(values, "source_type", "manual"))
            .bind(required_text(values, "calculation_version")?)
            .bind(spreadsheet_row_metadata(values))
            .execute(&mut **tx)
            .await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::ExternalEntityId => {
            let provider_id = payload_uuid(values, "_resolved_provider_id")?;
            let entity_type = required_text(values, "entity_type")?;
            let entity_id = if entity_type == "team" {
                resolve_committed_team_id(tx, values, context).await?
            } else {
                resolve_committed_id(values, &entity_type, context.player_keys, context.team_keys)?
            };
            write_external_entity_id(
                tx,
                &ExternalEntityIdDraft {
                    provider_id,
                    entity_type,
                    entity_id,
                    external_id: required_text(values, "external_id")?,
                    metadata: spreadsheet_row_metadata(values),
                },
            )
            .await?;
            Ok(if status == SpreadsheetRowStatus::ReadyUpdate {
                ApplyOutcome::Updated
            } else {
                ApplyOutcome::Inserted
            })
        }
    }
}

async fn resolve_or_create_import_team(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
) -> PersistenceResult<Uuid> {
    let name = required_text(values, "_auto_create_team_name")?;
    let normalized = normalize_name(&name);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(&normalized)
        .execute(&mut **tx)
        .await?;
    let matches = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT team.id
        FROM football.teams team
        WHERE team.normalized_name = $1
           OR EXISTS (
                SELECT 1
                FROM football.team_names alias
                WHERE alias.team_id = team.id
                  AND alias.normalized_name = $1
           )
        ORDER BY team.id
        LIMIT 2
        "#,
    )
    .bind(&normalized)
    .fetch_all(&mut **tx)
    .await?;
    match matches.as_slice() {
        [team_id] => Ok(*team_id),
        [] => {
            let team_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO football.teams (id,canonical_name,normalized_name,is_active,metadata) VALUES ($1,$2,$3,true,$4)",
            )
            .bind(team_id)
            .bind(&name)
            .bind(&normalized)
            .bind(json!({"source":"player_spreadsheet_auto_create","requires_profile_completion":true}))
            .execute(&mut **tx)
            .await?;
            crate::write_audit_event(
                tx,
                "team_created_from_player_import",
                "team",
                team_id.to_string(),
                json!({"canonical_name":name,"normalized_name":normalized}),
            )
            .await?;
            Ok(team_id)
        }
        _ => Err(PersistenceError::InvalidState(format!(
            "球队名称 {name} 匹配到多条记录，不能自动创建或关联"
        ))),
    }
}

async fn resolve_committed_team_id(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    context: &ImportCommitContext<'_>,
) -> PersistenceResult<Uuid> {
    if let Some(id) = payload_optional_uuid(values, "_resolved_team_id")? {
        return Ok(id);
    }
    if let Some(id) = optional_uuid(values, "team_id")? {
        return Ok(id);
    }
    let key = text(values, "_deferred_team_key");
    if !key.is_empty() {
        if let Some(id) = context.team_keys.get(&key) {
            return Ok(*id);
        }
        let normalized_key = normalize_name(&key);
        let key_matches = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT team.id
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            WHERE team.normalized_name = $1
               OR lower(btrim(COALESCE(profile.short_name, ''))) = $1
               OR EXISTS (
                    SELECT 1
                    FROM football.team_names alias
                    WHERE alias.team_id = team.id
                      AND alias.normalized_name = $1
               )
            ORDER BY team.id
            LIMIT 2
            "#,
        )
        .bind(&normalized_key)
        .fetch_all(&mut **tx)
        .await?;
        match key_matches.as_slice() {
            [team_id] => return Ok(*team_id),
            [] => {}
            _ => {
                return Err(PersistenceError::InvalidState(format!(
                    "完整资料包球队简称 {key} 匹配到多条记录，不能自动关联"
                )));
            }
        }
    }
    let name = ["_deferred_team_name", "team_name"]
        .into_iter()
        .map(|field| text(values, field))
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    if name.is_empty() {
        return Err(PersistenceError::InvalidState(format!(
            "无法解析导入关联：team {key}"
        )));
    }
    let normalized = normalize_name(&name);
    let matches = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT team.id
        FROM football.teams team
        WHERE team.normalized_name = $1
           OR EXISTS (
                SELECT 1
                FROM football.team_names alias
                WHERE alias.team_id = team.id
                  AND alias.normalized_name = $1
           )
        ORDER BY team.id
        LIMIT 2
        "#,
    )
    .bind(&normalized)
    .fetch_all(&mut **tx)
    .await?;
    match matches.as_slice() {
        [team_id] => Ok(*team_id),
        [] => Err(PersistenceError::InvalidState(format!(
            "完整资料包球队链提交后仍无法解析球队：{name}"
        ))),
        _ => Err(PersistenceError::InvalidState(format!(
            "球队名称 {name} 匹配到多条记录，不能自动关联"
        ))),
    }
}

async fn resolve_optional_committed_team_id(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    context: &ImportCommitContext<'_>,
) -> PersistenceResult<Option<Uuid>> {
    let has_reference = payload_optional_uuid(values, "_resolved_team_id")?.is_some()
        || optional_uuid(values, "team_id")?.is_some()
        || !text(values, "_deferred_team_key").is_empty()
        || !text(values, "_deferred_team_name").is_empty()
        || !text(values, "team_name").is_empty();
    if has_reference {
        resolve_committed_team_id(tx, values, context)
            .await
            .map(Some)
    } else {
        Ok(None)
    }
}

fn resolve_committed_id(
    values: &Map<String, Value>,
    prefix: &str,
    player_keys: &HashMap<String, Uuid>,
    team_keys: &HashMap<String, Uuid>,
) -> PersistenceResult<Uuid> {
    if let Some(id) = payload_optional_uuid(values, &format!("_resolved_{prefix}_id"))? {
        return Ok(id);
    }
    let key = text(values, &format!("_deferred_{prefix}_key"));
    let value = if prefix == "player" {
        player_keys.get(&key)
    } else {
        team_keys.get(&key)
    };
    value
        .copied()
        .ok_or_else(|| PersistenceError::InvalidState(format!("无法解析导入关联：{prefix} {key}")))
}

struct ImportCommitContext<'a> {
    player_keys: &'a mut HashMap<String, Uuid>,
    team_keys: &'a mut HashMap<String, Uuid>,
}
enum ApplyOutcome {
    Inserted,
    Updated,
}
