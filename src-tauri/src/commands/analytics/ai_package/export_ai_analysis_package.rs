use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_ai_analysis_package(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::AiAnalysisPackageSummary, String> {
    state
        .service
        .export_ai_analysis_package(output_path)
        .await
        .map_err(|error| error.to_string())
}
