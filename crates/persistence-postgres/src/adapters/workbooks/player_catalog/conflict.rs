use super::values::{payload_uuid, required_text};
use crate::adapters::workbooks::batch_ledger::{
    batch::{self as ledger, ImportFamily},
    mapping::{
        player_parse_action as parse_action, player_parse_entity_type as parse_entity_type,
        player_parse_import_mode as parse_import_mode,
    },
    rows as ledger_rows,
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    SpreadsheetAction, SpreadsheetConflictCandidate, SpreadsheetEntityType, SpreadsheetImportMode,
    SpreadsheetImportPreview, SpreadsheetImportResolution, SpreadsheetRowStatus,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn resolve_spreadsheet_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let mut tx = self.pool.begin().await?;
        let batch = ledger::lock_batch_in_tx(&mut tx, batch_id, ImportFamily::Player).await?;
        let batch_status: String = batch.try_get("status")?;
        ledger::require_pending(
            &batch_status,
            format!("导入批次状态为 {batch_status}，不能处理冲突"),
        )?;
        let import_mode = parse_import_mode(
            batch
                .try_get::<Option<String>, _>("import_mode")?
                .as_deref(),
        )?;
        let row = ledger_rows::lock_import_row_in_tx(&mut tx, batch_id, resolution.row_id).await?;
        let row_status: String = row.try_get("status")?;
        if row_status != "conflict" {
            return Err(PersistenceError::InvalidState(
                "只有冲突记录可以人工处理".to_string(),
            ));
        }
        if resolution.skip {
            ledger_rows::skip_import_row_in_tx(
                &mut tx,
                batch_id,
                resolution.row_id,
                "用户在预检中选择跳过",
            )
            .await?;
        } else {
            let selected = resolution.selected_entity_id.ok_or_else(|| {
                PersistenceError::InvalidState("请选择一个冲突候选记录".to_string())
            })?;
            let candidates_value: Value = row.try_get("conflict_candidates")?;
            let candidates: Vec<SpreadsheetConflictCandidate> =
                serde_json::from_value(candidates_value)?;
            if !candidates
                .iter()
                .any(|candidate| candidate.entity_id == selected)
            {
                return Err(PersistenceError::InvalidState(
                    "所选记录不属于该冲突的候选范围".to_string(),
                ));
            }
            let entity_type = parse_entity_type(&row.try_get::<String, _>("entity_type")?)?;
            let action = parse_action(&row.try_get::<String, _>("requested_action")?)?;
            let mut payload: Value = row.try_get("payload")?;
            let payload_object = payload.as_object_mut().ok_or_else(|| {
                PersistenceError::InvalidState("冲突记录内容不是对象".to_string())
            })?;
            let prefix = payload_object
                .get("_conflict_prefix")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| match entity_type {
                    SpreadsheetEntityType::Team => "team".to_string(),
                    _ => "player".to_string(),
                });
            payload_object.insert(format!("_resolved_{prefix}_id"), json!(selected));
            payload_object.remove("_conflict_prefix");
            let (status, message) = if entity_type == SpreadsheetEntityType::ExternalEntityId {
                let provider_id = payload_uuid(payload_object, "_resolved_provider_id")?;
                let external_entity_type = required_text(payload_object, "entity_type")?;
                let external_id = required_text(payload_object, "external_id")?;
                let existing_entity_id: Option<Uuid> = sqlx::query_scalar(
                    r#"
                    SELECT entity_id
                    FROM football.external_entity_ids
                    WHERE provider_id = $1 AND entity_type = $2 AND external_id = $3
                    "#,
                )
                .bind(provider_id)
                .bind(&external_entity_type)
                .bind(&external_id)
                .fetch_optional(&mut *tx)
                .await?;
                match existing_entity_id {
                    Some(existing) if existing != selected => {
                        return Err(PersistenceError::InvalidState(
                            "该外部 ID 已绑定到另一条数据库记录，禁止自动改绑".to_string(),
                        ));
                    }
                    Some(_)
                        if matches!(
                            action,
                            SpreadsheetAction::Update | SpreadsheetAction::Upsert
                        ) && import_mode == SpreadsheetImportMode::AddAndUpdate =>
                    {
                        (SpreadsheetRowStatus::ReadyUpdate, "已确认现有外部 ID 关联")
                    }
                    Some(_) => (SpreadsheetRowStatus::Skip, "相同外部 ID 关联已存在"),
                    None if action == SpreadsheetAction::Update => {
                        return Err(PersistenceError::InvalidState(
                            "标记为 update，但外部 ID 关联不存在".to_string(),
                        ));
                    }
                    None => (SpreadsheetRowStatus::ReadyAdd, "已选择外部 ID 关联记录"),
                }
            } else if matches!(
                entity_type,
                SpreadsheetEntityType::Team | SpreadsheetEntityType::Player
            ) {
                if matches!(
                    action,
                    SpreadsheetAction::Update
                        | SpreadsheetAction::Upsert
                        | SpreadsheetAction::Clear
                ) && import_mode == SpreadsheetImportMode::AddAndUpdate
                {
                    (
                        SpreadsheetRowStatus::ReadyUpdate,
                        "已关联现有记录，将执行更新",
                    )
                } else {
                    (
                        SpreadsheetRowStatus::Skip,
                        "已关联现有记录；当前动作不会更新",
                    )
                }
            } else {
                (SpreadsheetRowStatus::ReadyAdd, "已选择关联记录")
            };
            ledger_rows::resolve_import_row_in_tx(
                &mut tx,
                batch_id,
                ledger_rows::ResolvedImportRow {
                    id: resolution.row_id,
                    status: status.as_str(),
                    message: Some(message),
                    payload: &payload,
                    matched_entity_id: Some(selected),
                    candidates: &[],
                },
            )
            .await?;
        }
        crate::write_audit_event(
            &mut tx,
            "spreadsheet_import_conflict_resolved",
            "import_row",
            resolution.row_id.to_string(),
            json!({
                "batch_id": batch_id,
                "selected_entity_id": resolution.selected_entity_id,
                "skip": resolution.skip,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_spreadsheet_import_preview(batch_id).await
    }
}
