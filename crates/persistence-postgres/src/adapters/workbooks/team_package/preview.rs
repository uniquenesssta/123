use super::formation::validate_formation_group_rows;
use super::identity::canonical_team_import_action;
use super::values::{normalize_name, object, text};
use crate::adapters::workbooks::batch_ledger::{
    batch as ledger, mapping::team_import_mode_text, rows as ledger_rows,
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::{
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportMode, SpreadsheetImportPreview,
    SpreadsheetImportRow, SpreadsheetParsedWorkbook, SpreadsheetRowStatus, TEAM_MONTHLY_FORMAT,
};
use std::collections::HashSet;
use uuid::Uuid;

impl PostgresStore {
    pub async fn preview_team_monthly_import(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        if parsed.format_version != TEAM_MONTHLY_FORMAT {
            return Err(PersistenceError::InvalidState(format!(
                "球队月度工作簿版本错误：{}",
                parsed.format_version
            )));
        }
        if let Some(existing_id) =
            ledger::find_team_batch(&self.pool, &parsed.source_sha256).await?
        {
            return self.read_team_monthly_import_preview(existing_id).await;
        }

        let add_teams = parsed
            .rows
            .iter()
            .filter(|row| {
                row.entity_type == SpreadsheetEntityType::Team
                    && matches!(
                        row.action,
                        SpreadsheetAction::Add | SpreadsheetAction::Upsert
                    )
            })
            .filter_map(|row| text(object(&row.values).ok()?, "official_name"))
            .map(|value| normalize_name(&value))
            .collect::<HashSet<_>>();
        let add_coaches = parsed
            .rows
            .iter()
            .filter(|row| {
                row.entity_type == SpreadsheetEntityType::Coach
                    && matches!(
                        row.action,
                        SpreadsheetAction::Add | SpreadsheetAction::Upsert
                    )
            })
            .filter_map(|row| text(object(&row.values).ok()?, "official_name"))
            .map(|value| normalize_name(&value))
            .collect::<HashSet<_>>();

        let mut preview_rows = Vec::with_capacity(parsed.rows.len());
        for raw in &parsed.rows {
            let validation = self
                .validate_team_monthly_row(raw, mode, &add_teams, &add_coaches)
                .await;
            let row = match validation {
                Ok(validation) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: canonical_team_import_action(
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
                    action: canonical_team_import_action(
                        raw.action,
                        raw.entity_type,
                        SpreadsheetRowStatus::Error,
                    ),
                    status: SpreadsheetRowStatus::Error,
                    message: Some(error.to_string()),
                    payload: raw.values.clone(),
                    matched_entity_id: None,
                    conflict_candidates: vec![],
                },
            };
            preview_rows.push(row);
        }
        validate_formation_group_rows(&mut preview_rows);
        let counts = ledger_rows::count_preview_rows(&preview_rows);
        let batch_id = Uuid::new_v4();
        let mode_text = team_import_mode_text(mode);
        let mut tx = self.pool.begin().await?;
        ledger::create_team_batch_in_tx(&mut tx, batch_id, parsed, &counts, mode_text).await?;
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
