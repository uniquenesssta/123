use super::AppState;
use crate::openai_profiles::{
    OpenAiProfileDraft, OpenAiProfileSummary, OpenAiProfileTestResult, OpenAiProfilesState,
};
use football_research_gateway::{parse_api_example, ApiExampleParseResult, ApiProtocol};
use tauri::State;

#[tauri::command]
pub fn parse_openai_api_example(
    example: String,
    preferred_protocol: Option<ApiProtocol>,
) -> Result<ApiExampleParseResult, String> {
    parse_api_example(&example, preferred_protocol).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_openai_profiles(state: State<'_, AppState>) -> Result<OpenAiProfilesState, String> {
    state.openai_profiles.list()
}

#[tauri::command]
pub fn save_openai_profile(
    state: State<'_, AppState>,
    draft: OpenAiProfileDraft,
) -> Result<OpenAiProfileSummary, String> {
    state.openai_profiles.save(draft)
}

#[tauri::command]
pub fn set_active_openai_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<OpenAiProfilesState, String> {
    state.openai_profiles.set_active(&profile_id)
}

#[tauri::command]
pub fn delete_openai_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<OpenAiProfilesState, String> {
    state.openai_profiles.delete(&profile_id)
}

#[tauri::command]
pub fn clear_openai_profile_key(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<OpenAiProfilesState, String> {
    state.openai_profiles.clear_key(&profile_id)
}

#[tauri::command]
pub async fn test_openai_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<OpenAiProfileTestResult, String> {
    state.openai_profiles.test(&profile_id).await
}
