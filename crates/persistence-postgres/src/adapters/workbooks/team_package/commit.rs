use super::formation::{execute_formation_groups, normalize_formation_usage_payload};
use super::identity::{
    bind_batch_team_references, consolidate_duplicate_ready_add_team_rows,
    consolidate_duplicate_ready_add_team_rows_by_source,
};
use super::values::{
    normalize_monthly_datetime_payload, normalize_point_observation_window_payload,
    normalize_team_type_payload,
};
use super::write::execute_team_monthly_row;
use crate::adapters::workbooks::batch_ledger::{
    batch::{self as ledger, ImportFamily},
    mapping::team_import_row_from_db,
    rows as ledger_rows,
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{SpreadsheetEntityType, SpreadsheetImportCommitResult, SpreadsheetRowStatus};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn commit_team_monthly_import(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportCommitResult> {
        let mut tx = self.pool.begin().await?;
        let batch = ledger::lock_batch_in_tx(&mut tx, batch_id, ImportFamily::Team).await?;
        let batch_status: String = batch.try_get("status")?;
        if batch_status == "succeeded" {
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
        if batch_status != "pending" {
            return Err(PersistenceError::InvalidState(
                "导入批次状态不可提交".into(),
            ));
        }
        let blockers: i64 = ledger_rows::blocking_rows_in_tx(&mut tx, batch_id).await?;
        if blockers > 0 {
            return Err(PersistenceError::InvalidState(format!(
                "仍有 {blockers} 条冲突或错误记录"
            )));
        }
        ledger::start_batch_in_tx(&mut tx, batch_id).await?;

        let mut rows = ledger_rows::commit_rows_in_tx(&mut tx, batch_id, ImportFamily::Team)
            .await?
            .iter()
            .map(team_import_row_from_db)
            .collect::<PersistenceResult<Vec<_>>>()?;

        // Existing pending batches may have been previewed by an older client and can still
        // contain semantic aliases or Excel-rendered date/time strings. Canonicalize the stored
        // payload at the transaction boundary so retrying the same batch is safe after an upgrade.
        for row in rows.iter_mut() {
            let mut payload = row.payload.clone();
            let mut changed = normalize_monthly_datetime_payload(&mut payload)?;
            if matches!(
                row.entity_type,
                SpreadsheetEntityType::TeamTacticalObservation
                    | SpreadsheetEntityType::TeamAbilityObservation
            ) {
                changed |= normalize_point_observation_window_payload(&mut payload)?;
            }
            if row.entity_type == SpreadsheetEntityType::Team {
                changed |= normalize_team_type_payload(&mut payload)?;
            }
            if row.entity_type == SpreadsheetEntityType::FormationUsage {
                changed |= normalize_formation_usage_payload(&mut payload)?;
            }
            if changed {
                ledger_rows::set_import_payload_in_tx(&mut tx, row.id, &payload, None).await?;
                row.payload = payload;
            }
        }

        // Pending previews created by earlier clients may contain both an explicit
        // team overview row and an implicit club row derived from the player sheet.
        // Merge those rows before any insert so dependent rows always resolve one team.
        consolidate_duplicate_ready_add_team_rows(&mut tx, &mut rows).await?;
        consolidate_duplicate_ready_add_team_rows_by_source(&mut tx, &mut rows).await?;
        bind_batch_team_references(&mut tx, &mut rows).await?;

        let mut inserted = 0_u64;
        let mut updated = 0_u64;
        let mut ended_previous = 0_u64;
        let mut skipped = 0_u64;
        for row in rows
            .iter()
            .filter(|row| row.entity_type != SpreadsheetEntityType::FormationUsage)
        {
            if row.status == SpreadsheetRowStatus::Skip
                || row.status == SpreadsheetRowStatus::Imported
            {
                skipped += 1;
                continue;
            }
            let outcome = execute_team_monthly_row(&mut tx, row).await?;
            inserted += outcome.inserted;
            updated += outcome.updated;
            ended_previous += outcome.ended_previous;
            ledger_rows::mark_imported_in_tx(&mut tx, row.id).await?;
        }
        let formation_rows = rows
            .iter()
            .filter(|row| {
                row.entity_type == SpreadsheetEntityType::FormationUsage && row.status.is_ready()
            })
            .cloned()
            .collect::<Vec<_>>();
        if !formation_rows.is_empty() {
            let formation_inserted = execute_formation_groups(&mut tx, &formation_rows).await?;
            inserted += formation_inserted;
            for row in formation_rows {
                ledger_rows::mark_imported_in_tx(&mut tx, row.id).await?;
            }
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
        ledger::finish_batch_in_tx(&mut tx, &result, ImportFamily::Team).await?;
        tx.commit().await?;
        Ok(result)
    }
}
