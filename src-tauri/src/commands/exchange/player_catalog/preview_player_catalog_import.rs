use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn preview_player_catalog_import(
    state: State<'_, AppState>,
    input_path: String,
    mode: football_domain::SpreadsheetImportMode,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    state
        .service
        .preview_player_catalog_import(input_path, mode)
        .await
        .map_err(|error| error.to_string())
}
