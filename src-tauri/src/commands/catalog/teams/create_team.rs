use crate::bootstrap::AppState;
use football_domain::{TeamDraft, TeamRecord};
use tauri::State;

#[tauri::command]
pub async fn create_team(
    state: State<'_, AppState>,
    draft: TeamDraft,
) -> Result<TeamRecord, String> {
    state
        .service
        .create_team(draft)
        .await
        .map_err(|error| error.to_string())
}
