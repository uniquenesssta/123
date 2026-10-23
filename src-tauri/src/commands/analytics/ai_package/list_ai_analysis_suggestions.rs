use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_ai_analysis_suggestions(
    state: State<'_, AppState>,
    status: Option<String>,
    limit: u32,
) -> Result<Vec<football_domain::AiAnalysisSuggestionRecord>, String> {
    state
        .service
        .list_ai_analysis_suggestions(status, limit)
        .await
        .map_err(|error| error.to_string())
}
