use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn export_ai_analysis_package(
    state: State<'_, AppState>,
    output_path: String,
    expected_snapshot_id: Option<uuid::Uuid>,
) -> Result<football_domain::AiAnalysisPackageSummary, String> {
    let result = match expected_snapshot_id {
        Some(expected) => {
            state
                .service
                .export_ai_analysis_package_checked(output_path, expected)
                .await
        }
        None => state.service.export_ai_analysis_package(output_path).await,
    };
    result.map_err(|error| error.to_string())
}
