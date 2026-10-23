use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn preview_ai_match_package(
    state: State<'_, AppState>,
    input_path: String,
    mode: football_domain::SpreadsheetImportMode,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    state
        .service
        .preview_ai_match_package(input_path, mode)
        .await
        .map_err(|error| error.to_string())
}
