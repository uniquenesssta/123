use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub fn preview_ai_analysis_response(
    state: State<'_, AppState>,
    input_path: String,
) -> Result<football_domain::AiAnalysisResponsePreview, String> {
    state
        .service
        .preview_ai_analysis_response(input_path)
        .map_err(|error| error.to_string())
}
