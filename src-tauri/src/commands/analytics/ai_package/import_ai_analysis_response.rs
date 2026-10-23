use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn import_ai_analysis_response(
    state: State<'_, AppState>,
    input_path: String,
) -> Result<Vec<football_domain::AiAnalysisSuggestionRecord>, String> {
    state
        .service
        .import_ai_analysis_response(input_path)
        .await
        .map_err(|error| error.to_string())
}
