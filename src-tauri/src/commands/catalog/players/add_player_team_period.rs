use crate::bootstrap::AppState;
use football_domain::{PlayerTeamPeriodDraft, PlayerTeamPeriodRecord};
use tauri::State;

#[tauri::command]
pub async fn add_player_team_period(
    state: State<'_, AppState>,
    draft: PlayerTeamPeriodDraft,
) -> Result<PlayerTeamPeriodRecord, String> {
    state
        .service
        .add_player_team_period(draft)
        .await
        .map_err(|error| error.to_string())
}
