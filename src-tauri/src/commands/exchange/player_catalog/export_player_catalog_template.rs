use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_player_catalog_template(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::SpreadsheetExportSummary, String> {
    state
        .service
        .export_player_catalog_template(output_path)
        .await
        .map_err(|error| error.to_string())
}
