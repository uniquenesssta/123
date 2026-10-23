use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_team_package_preview_json(
    state: State<'_, AppState>,
    output_path: String,
    preview: football_domain::TeamPackageImportPreview,
) -> Result<football_domain::TeamPackagePreviewExportSummary, String> {
    state
        .service
        .export_team_package_preview_json(output_path, preview)
        .await
        .map_err(|error| error.to_string())
}
