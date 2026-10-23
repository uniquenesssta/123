use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn resolve_player_catalog_import_conflict(
    state: State<'_, AppState>,
    batch_id: String,
    resolution: football_domain::SpreadsheetImportResolution,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let parsed = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .resolve_player_catalog_import_conflict(parsed, resolution)
        .await
        .map_err(|error| error.to_string())
}
