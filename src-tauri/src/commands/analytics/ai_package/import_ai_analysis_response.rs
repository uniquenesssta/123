use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn import_ai_analysis_response(
    state: State<'_, AppState>,
    input_path: String,
    expected_preview: Option<football_domain::AiAnalysisResponsePreview>,
) -> Result<Vec<football_domain::AiAnalysisSuggestionRecord>, String> {
    let result = match expected_preview {
        Some(expected) => {
            state
                .service
                .import_ai_analysis_response_checked(input_path, expected)
                .await
        }
        None => state.service.import_ai_analysis_response(input_path).await,
    };
    result.map_err(|error| error.to_string())
}
