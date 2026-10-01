use super::super::batch_ledger::{
    batch::{self as ledger, ImportFamily},
    mapping::match_parse_entity as parse_entity,
    rows as ledger_rows,
};
use super::write::apply_match_exchange_row;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::SpreadsheetImportCommitResult;
use serde_json::Value;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn commit_match_lineup_import(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportCommitResult> {
        let mut tx = self.pool.begin().await?;
        let batch = ledger::lock_batch_in_tx(&mut tx, batch_id, ImportFamily::Match).await?;
        let status: String = batch.try_get("status")?;
        ledger::require_pending(&status, "导入批次不是待确认状态".to_string())?;
        let blocking: i64 = ledger_rows::blocking_rows_in_tx(&mut tx, batch_id).await?;
        if blocking > 0 {
            return Err(PersistenceError::InvalidState(
                "仍存在冲突或错误，不能导入".to_string(),
            ));
        }
        ledger::start_batch_in_tx(&mut tx, batch_id).await?;
        let rows = ledger_rows::commit_rows_in_tx(&mut tx, batch_id, ImportFamily::Match).await?;
        let mut match_keys = HashMap::new();
        let mut lineup_keys = HashMap::new();
        let mut inserted = 0u64;
        let mut updated = 0u64;
        let mut skipped = 0u64;
        let mut ended_previous = 0u64;
        let mut affected_lineups = Vec::<Uuid>::new();
        for row in rows {
            let row_id: Uuid = row.try_get("id")?;
            let row_status: String = row.try_get("status")?;
            if row_status == "skip" {
                skipped += 1;
                continue;
            }
            let entity = parse_entity(&row.try_get::<String, _>("entity_type")?)?;
            let payload: Value = row.try_get("payload")?;
            let values = payload
                .as_object()
                .ok_or_else(|| PersistenceError::InvalidState("导入内容无效".to_string()))?;
            let matched: Option<Uuid> = row.try_get("matched_entity_id")?;
            let outcome = apply_match_exchange_row(
                &mut tx,
                entity,
                values,
                matched,
                &mut match_keys,
                &mut lineup_keys,
            )
            .await?;
            if outcome.was_update {
                updated += 1
            } else {
                inserted += 1
            }
            ended_previous += outcome.ended_previous;
            if let Some(lineup_id) = outcome.lineup_id {
                affected_lineups.push(lineup_id);
            }
            ledger_rows::mark_imported_in_tx(&mut tx, row_id).await?;
        }
        affected_lineups.sort_unstable();
        affected_lineups.dedup();
        for lineup_id in affected_lineups {
            crate::adapters::lineups::chain::refresh_lineup_validation_in_tx(&mut tx, lineup_id)
                .await?;
        }
        let finished_at = Utc::now();
        let result = SpreadsheetImportCommitResult {
            batch_id,
            inserted_count: inserted,
            updated_count: updated,
            ended_previous_count: ended_previous,
            skipped_count: skipped,
            error_count: 0,
            finished_at,
        };
        ledger::finish_batch_in_tx(&mut tx, &result, ImportFamily::Match).await?;
        tx.commit().await?;
        Ok(result)
    }
}
