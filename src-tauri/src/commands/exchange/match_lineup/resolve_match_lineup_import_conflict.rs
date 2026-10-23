use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn resolve_match_lineup_import_conflict(
    state: State<'_, AppState>,
    batch_id: String,
    resolution: football_domain::SpreadsheetImportResolution,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let batch_id = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .resolve_match_lineup_import_conflict(batch_id, resolution)
        .await
        .map_err(|error| error.to_string())
}
