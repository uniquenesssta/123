use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_team_monthly_data(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::MonthlyWorkbookExportSummary, String> {
    state
        .service
        .export_team_monthly_data(output_path)
        .await
        .map_err(|error| error.to_string())
}
