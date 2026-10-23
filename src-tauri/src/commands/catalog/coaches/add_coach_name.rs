use crate::bootstrap::AppState;
use football_domain::{CoachNameDraft, CoachNameRecord};
use tauri::State;

#[tauri::command]
pub async fn add_coach_name(
    state: State<'_, AppState>,
    draft: CoachNameDraft,
) -> Result<CoachNameRecord, String> {
    state
        .service
        .add_coach_name(draft)
        .await
        .map_err(|error| error.to_string())
}
