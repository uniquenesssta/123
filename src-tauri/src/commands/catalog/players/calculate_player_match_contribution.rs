use crate::bootstrap::AppState;
use football_domain::{PlayerMatchContribution, PlayerMatchContributionRequest};
use tauri::State;

#[tauri::command]
pub async fn calculate_player_match_contribution(
    state: State<'_, AppState>,
    request: PlayerMatchContributionRequest,
) -> Result<PlayerMatchContribution, String> {
    state
        .service
        .calculate_player_match_contribution(request)
        .await
        .map_err(|error| error.to_string())
}
