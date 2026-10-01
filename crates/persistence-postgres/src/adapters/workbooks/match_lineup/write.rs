use super::{values::*, ApplyOutcome};
use crate::{
    role_resolution::{
        metadata_with_role_resolution, resolve_default_tactical_role_in_tx, resolve_tactical_role,
    },
    PersistenceError, PersistenceResult,
};
use chrono::{DateTime, Utc};
use football_domain::SpreadsheetEntityType;
use serde_json::{json, Map, Value};
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

pub(super) async fn apply_match_exchange_row(
    tx: &mut Transaction<'_, Postgres>,
    entity: SpreadsheetEntityType,
    values: &Map<String, Value>,
    matched: Option<Uuid>,
    match_keys: &mut HashMap<String, Uuid>,
    lineup_keys: &mut HashMap<String, Uuid>,
) -> PersistenceResult<ApplyOutcome> {
    match entity {
        SpreadsheetEntityType::Match => {
            let id = matched.unwrap_or_else(Uuid::new_v4);
            let match_key = required(values, "match_key")?;
            let competition_id = payload_uuid(values, "_resolved_competition_id")?;
            let home_team_id = payload_uuid(values, "_resolved_home_team_id")?;
            let away_team_id = payload_uuid(values, "_resolved_away_team_id")?;
            let status = default_text(values, "status", "scheduled");
            let metadata = json!({
                "source":"match_lineup_spreadsheet", "snapshot_type": optional_text(values,"snapshot_type"),
                "neutral_venue": optional_bool(values,"neutral_venue")?.unwrap_or(false),
                "weather": optional_text(values,"weather"), "surface": optional_text(values,"surface"),
                "importance": optional_text(values,"importance"), "tactical_notes": optional_text(values,"tactical_notes"),
                "travel_distance_home_km": optional_f64(values,"travel_distance_home_km")?,
                "travel_distance_away_km": optional_f64(values,"travel_distance_away_km")?,
                "rest_days_home": optional_i16(values,"rest_days_home")?, "rest_days_away": optional_i16(values,"rest_days_away")?,
                "schedule_density_home": optional_f64(values,"schedule_density_home")?, "schedule_density_away": optional_f64(values,"schedule_density_away")?,
            });
            if matched.is_some() {
                sqlx::query("UPDATE football.matches SET competition_id=$2,season_id=$3,stage_id=$4,round_id=$5,home_team_id=$6,away_team_id=$7,kickoff_time=$8,status=$9,venue=$10,metadata=metadata||$11,updated_at=now() WHERE id=$1")
                    .bind(id).bind(competition_id).bind(optional_uuid(values,"season_id")?).bind(optional_uuid(values,"stage_id")?).bind(optional_uuid(values,"round_id")?)
                    .bind(home_team_id).bind(away_team_id).bind(required_datetime(values,"kickoff_time")?).bind(status).bind(optional_text(values,"venue")).bind(metadata).execute(&mut **tx).await?;
            } else {
                sqlx::query("INSERT INTO football.matches(id,external_key,competition_id,season_id,stage_id,round_id,home_team_id,away_team_id,kickoff_time,status,venue,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)")
                    .bind(id).bind(&match_key).bind(competition_id).bind(optional_uuid(values,"season_id")?).bind(optional_uuid(values,"stage_id")?).bind(optional_uuid(values,"round_id")?)
                    .bind(home_team_id).bind(away_team_id).bind(required_datetime(values,"kickoff_time")?).bind(status).bind(optional_text(values,"venue")).bind(metadata).execute(&mut **tx).await?;
            }
            match_keys.insert(match_key, id);
            Ok(ApplyOutcome {
                was_update: matched.is_some(),
                ..ApplyOutcome::default()
            })
        }
        SpreadsheetEntityType::Lineup => {
            let match_id = resolve_import_match(values, match_keys)?;
            let team_id = payload_uuid(values, "_resolved_team_id")?;
            let lineup_type = required(values, "lineup_type")?;
            let snapshot_type = default_text(values, "snapshot_type", "T-1h");
            let _ =
                crate::adapters::lineups::pair_transaction::lock_match_in_tx(tx, match_id).await?;
            let supersedes_lineup_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM football.lineups WHERE match_id=$1 AND team_id=$2 AND snapshot_type=$3 AND lineup_type=$4 AND status='active' ORDER BY captured_at DESC,created_at DESC,id DESC LIMIT 1",
            )
            .bind(match_id)
            .bind(team_id)
            .bind(&snapshot_type)
            .bind(&lineup_type)
            .fetch_optional(&mut **tx)
            .await?;
            let superseded = sqlx::query(
                "UPDATE football.lineups SET status='superseded',updated_at=now() WHERE match_id=$1 AND team_id=$2 AND snapshot_type=$3 AND lineup_type=$4 AND status='active'",
            )
            .bind(match_id)
            .bind(team_id)
            .bind(&snapshot_type)
            .bind(&lineup_type)
            .execute(&mut **tx)
            .await?
            .rows_affected();
            let id = Uuid::new_v4();
            sqlx::query(
                r#"INSERT INTO football.lineups(
                    id,match_id,team_id,lineup_type,snapshot_type,formation,formation_id,
                    coach_id,captured_at,status,quality_score,source_urls,
                    supersedes_lineup_id,metadata
                ) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,'active',$10,$11,$12,$13)"#,
            )
            .bind(id)
            .bind(match_id)
            .bind(team_id)
            .bind(lineup_type)
            .bind(snapshot_type)
            .bind(optional_text(values, "formation"))
            .bind(payload_optional_uuid(values, "_resolved_formation_id")?)
            .bind(optional_uuid(values, "coach_id")?)
            .bind(required_datetime(values, "captured_at")?)
            .bind(optional_f64(values, "quality_score")?)
            .bind(parse_source_urls(values, "source_urls"))
            .bind(supersedes_lineup_id)
            .bind(
                json!({"source":"match_lineup_spreadsheet","notes":optional_text(values,"notes")}),
            )
            .execute(&mut **tx)
            .await?;
            lineup_keys.insert(required(values, "lineup_key")?, id);
            Ok(ApplyOutcome {
                was_update: false,
                ended_previous: superseded,
                lineup_id: Some(id),
            })
        }
        SpreadsheetEntityType::LineupPlayer => {
            let lineup_key = required(values, "lineup_key")?;
            let lineup_id = lineup_keys
                .get(&lineup_key)
                .copied()
                .or(optional_uuid(values, "lineup_id")?)
                .ok_or_else(|| {
                    PersistenceError::InvalidState(format!("无法找到阵容：{lineup_key}"))
                })?;
            let player_id = payload_uuid(values, "_resolved_player_id")?;
            let (lineup_match_id, lineup_team_id, lineup_captured_at): (Uuid, Uuid, DateTime<Utc>) =
                sqlx::query_as(
                    "SELECT match_id,team_id,captured_at FROM football.lineups WHERE id=$1",
                )
                .bind(lineup_id)
                .fetch_one(&mut **tx)
                .await?;
            let requested_match_id = resolve_import_match(values, match_keys)?;
            let requested_team_id = payload_uuid(values, "_resolved_team_id")?;
            if lineup_match_id != requested_match_id || lineup_team_id != requested_team_id {
                return Err(PersistenceError::InvalidState(
                    "阵容球员与 lineup_key 指向的比赛或球队不一致".to_string(),
                ));
            }
            let position_code = optional_text(values, "position_code")
                .map(|value| value.trim().to_uppercase())
                .filter(|value| !value.is_empty());
            let inherited_role = resolve_default_tactical_role_in_tx(
                tx,
                player_id,
                position_code.as_deref(),
                lineup_captured_at.date_naive(),
            )
            .await?;
            let role_resolution = resolve_tactical_role(
                optional_text(values, "role_code").as_deref(),
                inherited_role.as_ref(),
            );
            let metadata = metadata_with_role_resolution(
                &json!({
                    "source": "match_lineup_spreadsheet",
                    "notes": optional_text(values, "notes")
                }),
                &role_resolution,
            );
            sqlx::query(r#"INSERT INTO football.lineup_players(
                    lineup_id,player_id,position_code,role_code,is_starter,shirt_number,
                    expected_minutes,actual_minutes,sequence_no,bench_order,
                    availability_status,starting_probability,membership_override,source_urls,metadata
                ) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
                ON CONFLICT(lineup_id,player_id) DO UPDATE SET
                    position_code=EXCLUDED.position_code,role_code=EXCLUDED.role_code,
                    is_starter=EXCLUDED.is_starter,shirt_number=EXCLUDED.shirt_number,
                    expected_minutes=EXCLUDED.expected_minutes,actual_minutes=EXCLUDED.actual_minutes,
                    sequence_no=EXCLUDED.sequence_no,bench_order=EXCLUDED.bench_order,
                    availability_status=EXCLUDED.availability_status,
                    starting_probability=EXCLUDED.starting_probability,
                    membership_override=EXCLUDED.membership_override,
                    source_urls=EXCLUDED.source_urls,metadata=EXCLUDED.metadata"#)
                .bind(lineup_id).bind(player_id).bind(position_code).bind(role_resolution.role_code.as_deref())
                .bind(required_bool(values,"is_starter")?).bind(optional_i16(values,"shirt_number")?).bind(optional_i16(values,"expected_minutes")?).bind(optional_i16(values,"actual_minutes")?)
                .bind(optional_i16(values,"sequence_no")?.unwrap_or(0)).bind(optional_i16(values,"bench_order")?)
                .bind(optional_text(values,"availability_status")).bind(optional_f64(values,"starting_probability")?)
                .bind(optional_bool(values,"membership_override")?.unwrap_or(false)).bind(parse_source_urls(values,"source_urls"))
                .bind(metadata).execute(&mut **tx).await?;
            Ok(ApplyOutcome {
                lineup_id: Some(lineup_id),
                ..ApplyOutcome::default()
            })
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            let player_id = payload_uuid(values, "_resolved_player_id")?;
            sqlx::query("INSERT INTO feature.player_dynamic_tags(id,player_id,tag_code,value,label,confidence,observed_at,valid_from,valid_to,competition_id,position_code,opponent_team_id,sample_size,source_type,calculation_version,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)")
                .bind(Uuid::new_v4()).bind(player_id).bind(required(values,"tag_code")?).bind(required_f64(values,"tag_value")?).bind(optional_text(values,"label"))
                .bind(optional_f64(values,"confidence")?.unwrap_or(1.0)).bind(required_datetime(values,"observed_at")?).bind(required_datetime(values,"valid_from")?).bind(required_datetime(values,"valid_to")?)
                .bind(optional_uuid(values,"competition_id")?).bind(optional_text(values,"position_code").map(|value|value.to_uppercase())).bind(optional_uuid(values,"opponent_team_id")?)
                .bind(optional_i32(values,"sample_size")?.unwrap_or(1)).bind(default_text(values,"source_type","lineup_import")).bind(required(values,"calculation_version")?).bind(json!({"source":"match_lineup_spreadsheet"})).execute(&mut **tx).await?;
            Ok(ApplyOutcome::default())
        }
        _ => Err(PersistenceError::InvalidState(
            "不支持的比赛导入实体".to_string(),
        )),
    }
}
