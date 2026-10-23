use crate::bootstrap::AppState;
use crate::openai_profiles::OpenAiProfilesState;
use tauri::State;

#[tauri::command]
pub fn delete_openai_profile(
    state: State<'_, AppState>,
    profile_id: String,
) -> Result<OpenAiProfilesState, String> {
    state.openai_profiles.delete(&profile_id)
}
