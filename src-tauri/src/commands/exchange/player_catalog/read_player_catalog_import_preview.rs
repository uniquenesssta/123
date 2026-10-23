use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn read_player_catalog_import_preview(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let parsed = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .read_player_catalog_import_preview(parsed)
        .await
        .map_err(|error| error.to_string())
}
