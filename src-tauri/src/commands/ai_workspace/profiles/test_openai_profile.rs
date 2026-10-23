use crate::bootstrap::AppState;
use crate::openai_profiles::OpenAiProfileTestResult;
use tauri::State;

#[tauri::command]
pub async fn test_openai_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<OpenAiProfileTestResult, String> {
    state.openai_profiles.test(&profile_id).await
}
