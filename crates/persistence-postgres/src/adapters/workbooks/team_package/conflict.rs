use super::values::object_mut;
use crate::adapters::workbooks::batch_ledger::{
    batch::{self as ledger, ImportFamily},
    rows as ledger_rows,
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    SpreadsheetConflictCandidate, SpreadsheetImportPreview, SpreadsheetImportResolution,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn resolve_team_monthly_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let mut tx = self.pool.begin().await?;
        let status: String = ledger::lock_batch_in_tx(&mut tx, batch_id, ImportFamily::Team)
            .await?
            .try_get("status")?;
        if status != "pending" {
            return Err(PersistenceError::InvalidState(
                "导入批次已不允许处理冲突".into(),
            ));
        }
        let row = ledger_rows::lock_import_row_in_tx(&mut tx, batch_id, resolution.row_id).await?;
        if row.try_get::<String, _>("status")? != "conflict" {
            return Err(PersistenceError::InvalidState("只有冲突行可处理".into()));
        }
        if resolution.skip {
            ledger_rows::skip_import_row_in_tx(
                &mut tx,
                batch_id,
                resolution.row_id,
                "用户选择跳过",
            )
            .await?;
        } else {
            let selected = resolution
                .selected_entity_id
                .ok_or_else(|| PersistenceError::InvalidState("请选择候选记录".into()))?;
            let candidates: Vec<SpreadsheetConflictCandidate> =
                serde_json::from_value(row.try_get("conflict_candidates")?)?;
            if !candidates
                .iter()
                .any(|candidate| candidate.entity_id == selected)
            {
                return Err(PersistenceError::InvalidState(
                    "所选记录不在候选范围".into(),
                ));
            }
            let mut payload: Value = row.try_get("payload")?;
            let object = object_mut(&mut payload)?;
            let prefix = object
                .get("_conflict_prefix")
                .and_then(Value::as_str)
                .unwrap_or("entity")
                .to_string();
            object.insert(format!("_resolved_{prefix}_id"), json!(selected));
            object.remove("_conflict_prefix");
            ledger_rows::resolve_import_row_in_tx(
                &mut tx,
                batch_id,
                ledger_rows::ResolvedImportRow {
                    id: resolution.row_id,
                    status: "ready_update",
                    message: Some("已人工选择唯一记录"),
                    payload: &payload,
                    matched_entity_id: Some(selected),
                    candidates: &[],
                },
            )
            .await?;
        }
        tx.commit().await?;
        self.read_team_monthly_import_preview(batch_id).await
    }
}
