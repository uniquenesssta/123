use crate::bootstrap::AppState;
use crate::openai_profiles::{OpenAiProfileDraft, OpenAiProfileSummary};
use tauri::State;

#[tauri::command]
pub fn save_openai_profile(
    state: State<'_, AppState>,
    draft: OpenAiProfileDraft,
) -> Result<OpenAiProfileSummary, String> {
    state.openai_profiles.save(draft)
}
