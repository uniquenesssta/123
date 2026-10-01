use super::super::batch_ledger::{
    batch::{self as ledger, ImportFamily},
    mapping::{
        match_parse_action as parse_action, match_parse_entity as parse_entity,
        match_parse_mode as parse_mode,
    },
    rows as ledger_rows,
};
use super::identity::error;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    SpreadsheetConflictCandidate, SpreadsheetEntityType, SpreadsheetImportPreview,
    SpreadsheetImportResolution,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn resolve_match_lineup_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let (entity, action, mode, payload) = {
            let mut tx = self.pool.begin().await?;
            let batch = ledger::lock_batch_in_tx(&mut tx, batch_id, ImportFamily::Match).await?;
            let batch_status: String = batch.try_get("status")?;
            ledger::require_pending(&batch_status, "该导入批次已不能修改".to_string())?;
            let mode = parse_mode(
                batch
                    .try_get::<Option<String>, _>("import_mode")?
                    .as_deref(),
            )?;
            let row =
                ledger_rows::lock_import_row_in_tx(&mut tx, batch_id, resolution.row_id).await?;
            let status: String = row.try_get("status")?;
            if status != "conflict" {
                return Err(PersistenceError::InvalidState(
                    "只有冲突行可以处理".to_string(),
                ));
            }
            if resolution.skip {
                ledger_rows::skip_import_row_in_tx(
                    &mut tx,
                    batch_id,
                    resolution.row_id,
                    "用户选择跳过",
                )
                .await?;
                tx.commit().await?;
                return self.read_match_lineup_import_preview(batch_id).await;
            }

            let selected = resolution
                .selected_entity_id
                .ok_or_else(|| PersistenceError::InvalidState("请选择候选记录".to_string()))?;
            let candidates: Vec<SpreadsheetConflictCandidate> =
                serde_json::from_value(row.try_get("conflict_candidates")?)?;
            if !candidates.iter().any(|item| item.entity_id == selected) {
                return Err(PersistenceError::InvalidState(
                    "所选记录不在冲突候选中".to_string(),
                ));
            }
            let entity = parse_entity(&row.try_get::<String, _>("entity_type")?)?;
            let action = parse_action(&row.try_get::<String, _>("requested_action")?)?;
            let mut payload: Value = row.try_get("payload")?;
            let object = payload
                .as_object_mut()
                .ok_or_else(|| PersistenceError::InvalidState("导入行无效".to_string()))?;
            let prefix = object
                .get("_conflict_prefix")
                .and_then(Value::as_str)
                .unwrap_or(match entity {
                    SpreadsheetEntityType::Match => "match",
                    SpreadsheetEntityType::Lineup => "team",
                    SpreadsheetEntityType::LineupPlayer
                    | SpreadsheetEntityType::PlayerDynamicTag => "player",
                    _ => "entity",
                });
            let target_field = match prefix {
                "match" => "match_id",
                "home_team" => "home_team_id",
                "away_team" => "away_team_id",
                "team" => "team_id",
                "player" => "player_id",
                _ => {
                    return Err(PersistenceError::InvalidState(format!(
                        "未知冲突类型：{prefix}"
                    )))
                }
            };
            object.insert(target_field.to_string(), json!(selected));
            object.remove("_conflict_prefix");
            tx.commit().await?;
            (entity, action, mode, payload)
        };

        let validation = match self
            .validate_match_exchange_row(entity, action, &payload, mode)
            .await
        {
            Ok(validation) => validation,
            Err(error_value) => error(payload, error_value.to_string()),
        };

        let mut tx = self.pool.begin().await?;
        let batch_status: String = ledger::lock_batch_in_tx(&mut tx, batch_id, ImportFamily::Match)
            .await?
            .try_get("status")?;
        ledger::require_pending(&batch_status, "该导入批次已不能修改".to_string())?;
        let row_status: String =
            ledger_rows::lock_import_row_in_tx(&mut tx, batch_id, resolution.row_id)
                .await?
                .try_get("status")?;
        if row_status != "conflict" {
            return Err(PersistenceError::InvalidState(
                "该冲突行已被其他操作修改".to_string(),
            ));
        }
        ledger_rows::resolve_import_row_in_tx(
            &mut tx,
            batch_id,
            ledger_rows::ResolvedImportRow {
                id: resolution.row_id,
                status: validation.status.as_str(),
                message: validation.message.as_deref(),
                payload: &validation.payload,
                matched_entity_id: validation.matched_entity_id,
                candidates: &validation.candidates,
            },
        )
        .await?;
        tx.commit().await?;
        self.read_match_lineup_import_preview(batch_id).await
    }
}
