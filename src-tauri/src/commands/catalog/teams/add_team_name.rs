use crate::bootstrap::AppState;
use football_domain::{TeamNameDraft, TeamNameRecord};
use tauri::State;

#[tauri::command]
pub async fn add_team_name(
    state: State<'_, AppState>,
    draft: TeamNameDraft,
) -> Result<TeamNameRecord, String> {
    state
        .service
        .add_team_name(draft)
        .await
        .map_err(|error| error.to_string())
}
