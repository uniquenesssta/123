use crate::bootstrap::AppState;
use crate::openai_profiles::OpenAiProfilesState;
use tauri::State;

#[tauri::command]
pub fn list_openai_profiles(state: State<'_, AppState>) -> Result<OpenAiProfilesState, String> {
    state.openai_profiles.list()
}
