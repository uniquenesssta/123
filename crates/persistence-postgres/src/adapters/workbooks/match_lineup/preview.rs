use super::super::batch_ledger::{batch as ledger, rows as ledger_rows};
use super::super::identity::row::validate_workbook_row_locations;
use crate::{PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::{
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportRow,
    SpreadsheetParsedWorkbook, SpreadsheetRowStatus,
};
use uuid::Uuid;

impl PostgresStore {
    pub async fn preview_match_lineup_import(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        validate_workbook_row_locations(parsed)?;
        let batch_id = Uuid::new_v4();
        let mut preview_rows = Vec::with_capacity(parsed.rows.len());
        for raw in &parsed.rows {
            let validation = self
                .validate_match_exchange_row(raw.entity_type, raw.action, &raw.values, mode)
                .await;
            let row = match validation {
                Ok(validation) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: raw.action,
                    status: validation.status,
                    message: validation.message,
                    payload: validation.payload,
                    matched_entity_id: validation.matched_entity_id,
                    conflict_candidates: validation.candidates,
                },
                Err(error) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: raw.action,
                    status: SpreadsheetRowStatus::Error,
                    message: Some(error.to_string()),
                    payload: raw.values.clone(),
                    matched_entity_id: None,
                    conflict_candidates: Vec::new(),
                },
            };
            preview_rows.push(row);
        }
        let counts = ledger_rows::count_preview_rows(&preview_rows);
        let mut tx = self.pool.begin().await?;
        ledger::create_match_batch_in_tx(&mut tx, batch_id, parsed, &counts, mode).await?;
        for row in &preview_rows {
            ledger_rows::insert_import_row(&mut tx, batch_id, row).await?;
        }
        tx.commit().await?;
        Ok(SpreadsheetImportPreview {
            batch_id,
            source_file_name: parsed.source_file_name.clone(),
            source_sha256: parsed.source_sha256.clone(),
            import_mode: mode,
            counts,
            rows: preview_rows,
            created_at: Utc::now(),
        })
    }
}
