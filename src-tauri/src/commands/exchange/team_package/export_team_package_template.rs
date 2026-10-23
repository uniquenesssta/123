use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_team_package_template(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::TeamPackageExportSummary, String> {
    state
        .service
        .export_team_package_template(output_path)
        .await
        .map_err(|error| error.to_string())
}
