use super::identity::{canonical_spreadsheet_import_action, collect_keys, duplicate_keys};
use super::SpreadsheetValidationContext;
use crate::adapters::workbooks::batch_ledger::{batch as ledger, rows as ledger_rows};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::{
    SpreadsheetEntityType, SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportRow,
    SpreadsheetParsedWorkbook, SpreadsheetRowStatus, PLAYER_IMPORT_FORMAT, PLAYER_MONTHLY_FORMAT,
};
use serde_json::json;
use sqlx::Row;
use std::collections::HashMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn preview_spreadsheet_import(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let team_references = HashMap::new();
        self.preview_spreadsheet_import_inner(parsed, mode, &team_references)
            .await
    }

    pub async fn preview_spreadsheet_import_with_team_references(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
        team_references: &HashMap<String, String>,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        self.preview_spreadsheet_import_inner(parsed, mode, team_references)
            .await
    }

    async fn preview_spreadsheet_import_inner(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
        external_team_references: &HashMap<String, String>,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let import_type = player_import_type(&parsed.format_version)?;
        if let Some(existing) =
            ledger::find_player_batch(&self.pool, &parsed.source_sha256, import_type).await?
        {
            let existing_id: Uuid = existing.try_get("id")?;
            let existing_status: String = existing.try_get("status")?;
            if existing_status != "pending" {
                return self.read_spreadsheet_import_preview(existing_id).await;
            }
            ledger::cancel_pending_player_batch(&self.pool,existing_id,json!({"cancel_reason":"repreview_same_source","replacement_requested_at":Utc::now()})).await?;
        }
        let batch_id = Uuid::new_v4();
        let mode_text = match mode {
            SpreadsheetImportMode::AddOnly => "add_only",
            SpreadsheetImportMode::AddAndUpdate => "add_and_update",
        };
        let player_keys = collect_keys(&parsed.rows, SpreadsheetEntityType::Player, "player_key");
        let team_keys = collect_keys(&parsed.rows, SpreadsheetEntityType::Team, "team_key");
        let duplicate_player_keys =
            duplicate_keys(&parsed.rows, SpreadsheetEntityType::Player, "player_key");
        let duplicate_team_keys =
            duplicate_keys(&parsed.rows, SpreadsheetEntityType::Team, "team_key");
        let validation_context = SpreadsheetValidationContext {
            mode,
            player_keys: &player_keys,
            team_keys: &team_keys,
            duplicate_player_keys: &duplicate_player_keys,
            duplicate_team_keys: &duplicate_team_keys,
            external_team_references,
        };

        let mut preview_rows = Vec::with_capacity(parsed.rows.len());
        for raw in &parsed.rows {
            let validation = self
                .validate_spreadsheet_row(
                    raw.entity_type,
                    raw.action,
                    &raw.values,
                    &validation_context,
                )
                .await;
            let row = match validation {
                Ok(validation) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: canonical_spreadsheet_import_action(
                        raw.action,
                        raw.entity_type,
                        validation.status,
                    ),
                    status: validation.status,
                    message: validation.message,
                    payload: validation.payload,
                    matched_entity_id: validation.matched_entity_id,
                    conflict_candidates: validation.conflict_candidates,
                },
                Err(error) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: canonical_spreadsheet_import_action(
                        raw.action,
                        raw.entity_type,
                        SpreadsheetRowStatus::Error,
                    ),
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
        ledger::create_player_batch_in_tx(
            &mut tx,
            batch_id,
            parsed,
            &counts,
            import_type,
            mode_text,
        )
        .await?;
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
fn player_import_type(format_version: &str) -> PersistenceResult<&'static str> {
    match format_version {
        PLAYER_IMPORT_FORMAT => Ok("player_catalog_xlsx"),
        PLAYER_MONTHLY_FORMAT => Ok("player_monthly_xlsx"),
        other => Err(PersistenceError::InvalidState(format!(
            "不支持的球员工作簿版本：{other}"
        ))),
    }
}
