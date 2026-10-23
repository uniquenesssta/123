use crate::bootstrap::AppState;
use crate::openai_profiles::OpenAiProfilesState;
use tauri::State;

#[tauri::command]
pub fn clear_openai_profile_key(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<OpenAiProfilesState, String> {
    state.openai_profiles.clear_key(&profile_id)
}
