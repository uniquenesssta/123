use crate::bootstrap::AppState;
use football_domain::{CoachDraft, CoachRecord};
use tauri::State;

#[tauri::command]
pub async fn create_coach(
    state: State<'_, AppState>,
    draft: CoachDraft,
) -> Result<CoachRecord, String> {
    state
        .service
        .create_coach(draft)
        .await
        .map_err(|error| error.to_string())
}
