use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn decide_ai_analysis_suggestion(
    state: State<'_, AppState>,
    draft: football_domain::AiSuggestionDecisionDraft,
) -> Result<football_domain::AiAnalysisSuggestionRecord, String> {
    state
        .service
        .decide_ai_analysis_suggestion(draft)
        .await
        .map_err(|error| error.to_string())
}
