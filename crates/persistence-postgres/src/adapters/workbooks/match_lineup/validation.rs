use super::{identity::*, values::*, Validation};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportMode};
use serde_json::{json, Map, Value};
use sqlx::Row;

impl PostgresStore {
    pub(super) async fn validate_match_exchange_row(
        &self,
        entity: SpreadsheetEntityType,
        action: SpreadsheetAction,
        values: &Value,
        mode: SpreadsheetImportMode,
    ) -> PersistenceResult<Validation> {
        if action == SpreadsheetAction::Skip {
            return Ok(skip(values.clone(), "Excel 行标记为 skip"));
        }
        let mut payload = values
            .as_object()
            .cloned()
            .ok_or_else(|| PersistenceError::InvalidState("Excel 行内容不是对象".to_string()))?;
        match entity {
            SpreadsheetEntityType::Match => {
                required(&payload, "match_key")?;
                required(&payload, "kickoff_time")?;
                let competition = resolve_competition(&self.pool, &payload).await?;
                payload.insert("_resolved_competition_id".to_string(), json!(competition));
                let home =
                    resolve_team_value(&self.pool, &payload, "home_team_id", "home_team_name")
                        .await?;
                if let Some(result) = resolution_validation(&mut payload, home, "home_team") {
                    return Ok(result);
                }
                let away =
                    resolve_team_value(&self.pool, &payload, "away_team_id", "away_team_name")
                        .await?;
                if let Some(result) = resolution_validation(&mut payload, away, "away_team") {
                    return Ok(result);
                }
                if payload_uuid(&payload, "_resolved_home_team_id")?
                    == payload_uuid(&payload, "_resolved_away_team_id")?
                {
                    return Ok(error(Value::Object(payload), "主客队不能相同"));
                }
                required_datetime(&payload, "kickoff_time")?;
                let existing = find_match(&self.pool, &payload).await?;
                decision(action, mode, Value::Object(payload), existing, "match")
            }
            SpreadsheetEntityType::Lineup => {
                required(&payload, "lineup_key")?;
                required(&payload, "match_key")?;
                validate_lineup_type(&payload)?;
                let snapshot_type = default_text(&payload, "snapshot_type", "T-1h");
                crate::adapters::lineups::chain::normalize_lineup_snapshot_type(&snapshot_type)?;
                payload.insert("snapshot_type".to_string(), json!(snapshot_type));
                required_datetime(&payload, "captured_at")?;
                let team = resolve_team_value(&self.pool, &payload, "team_id", "team_name").await?;
                if let Some(result) = resolution_validation(&mut payload, team, "team") {
                    return Ok(result);
                }
                resolve_match_reference(&self.pool, &mut payload).await?;
                validate_lineup_match_team(&self.pool, &payload).await?;
                resolve_lineup_formation(&self.pool, &mut payload).await?;
                if let Some(coach_id) = optional_uuid(&payload, "coach_id")? {
                    let exists: bool = sqlx::query_scalar(
                        "SELECT EXISTS(SELECT 1 FROM football.coaches WHERE id=$1 AND status='active')",
                    )
                    .bind(coach_id)
                    .fetch_one(&self.pool)
                    .await?;
                    if !exists {
                        return Ok(error(Value::Object(payload), "coach_id 不存在或已归档"));
                    }
                }
                Ok(ready(
                    Value::Object(payload),
                    "阵容版本已验证；提交后执行完整性门禁",
                ))
            }
            SpreadsheetEntityType::LineupPlayer => {
                required(&payload, "lineup_key")?;
                required(&payload, "match_key")?;
                let player = resolve_player_value(&self.pool, &payload).await?;
                if let Some(result) = resolution_validation(&mut payload, player, "player") {
                    return Ok(result);
                }
                validate_lineup_player(&payload)?;
                let team = resolve_team_value(&self.pool, &payload, "team_id", "team_name").await?;
                if let Some(result) = resolution_validation(&mut payload, team, "team") {
                    return Ok(result);
                }
                resolve_match_reference(&self.pool, &mut payload).await?;
                validate_lineup_match_team(&self.pool, &payload).await?;
                Ok(ready(
                    Value::Object(payload),
                    "阵容球员已验证；球队履历将在提交时校验",
                ))
            }
            SpreadsheetEntityType::PlayerDynamicTag => {
                let player = resolve_player_value(&self.pool, &payload).await?;
                if let Some(result) = resolution_validation(&mut payload, player, "player") {
                    return Ok(result);
                }
                validate_dynamic_tag(&self.pool, &payload).await?;
                Ok(ready(Value::Object(payload), "动态标签已验证"))
            }
            _ => Ok(error(
                Value::Object(payload),
                "该工作表不属于比赛与阵容模板",
            )),
        }
    }
}

pub(super) fn validate_lineup_type(values: &Map<String, Value>) -> PersistenceResult<()> {
    let value = required(values, "lineup_type")?;
    if matches!(value.as_str(), "expected" | "confirmed" | "actual") {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(
            "lineup_type 无效".to_string(),
        ))
    }
}

pub(super) fn validate_lineup_player(values: &Map<String, Value>) -> PersistenceResult<()> {
    let is_starter = required_bool(values, "is_starter")?;
    if let Some(v) = optional_i16(values, "shirt_number")? {
        if !(0..=99).contains(&v) {
            return Err(PersistenceError::InvalidState(
                "球衣号码必须为 0–99".to_string(),
            ));
        }
    }
    for field in ["expected_minutes", "actual_minutes"] {
        if let Some(v) = optional_i16(values, field)? {
            if !(0..=150).contains(&v) {
                return Err(PersistenceError::InvalidState(format!(
                    "{field} 必须为 0–150"
                )));
            }
        }
    }
    if let Some(probability) = optional_f64(values, "starting_probability")? {
        if !(0.0..=1.0).contains(&probability) {
            return Err(PersistenceError::InvalidState(
                "starting_probability 必须位于 0–1".to_string(),
            ));
        }
    }
    if let Some(order) = optional_i16(values, "bench_order")? {
        if !(1..=99).contains(&order) {
            return Err(PersistenceError::InvalidState(
                "bench_order 必须位于 1–99".to_string(),
            ));
        }
        if is_starter {
            return Err(PersistenceError::InvalidState(
                "首发球员不能设置 bench_order".to_string(),
            ));
        }
    }
    if let Some(status) = optional_text(values, "availability_status") {
        availability_from_str(&status)?;
    }
    optional_bool(values, "membership_override")?;
    Ok(())
}

pub(super) async fn validate_dynamic_tag(
    pool: &sqlx::PgPool,
    values: &Map<String, Value>,
) -> PersistenceResult<()> {
    let code = required(values, "tag_code")?;
    let row=sqlx::query("SELECT minimum_value,maximum_value FROM feature.player_dynamic_tag_definitions WHERE code=$1").bind(&code).fetch_optional(pool).await?.ok_or_else(||PersistenceError::InvalidState(format!("动态标签不存在：{code}")))?;
    let value = required_f64(values, "tag_value")?;
    let min: f64 = row.try_get("minimum_value")?;
    let max: f64 = row.try_get("maximum_value")?;
    if value < min || value > max {
        return Err(PersistenceError::InvalidState(format!(
            "标签值超出允许范围 {min}–{max}"
        )));
    }
    let from = required_datetime(values, "valid_from")?;
    let to = required_datetime(values, "valid_to")?;
    if to <= from {
        return Err(PersistenceError::InvalidState(
            "动态标签失效时间必须晚于生效时间".to_string(),
        ));
    }
    Ok(())
}
