use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_match_lineup_template(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::MatchLineupExportSummary, String> {
    state
        .service
        .export_match_lineup_template(output_path)
        .await
        .map_err(|error| error.to_string())
}
