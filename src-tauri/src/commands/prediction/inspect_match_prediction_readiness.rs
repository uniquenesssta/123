use crate::bootstrap::AppState;
use football_application::StoredMatchPredictionCommand;
use football_domain::MatchPredictionReadiness;
use tauri::State;

#[tauri::command]
pub async fn inspect_match_prediction_readiness(
    state: State<'_, AppState>,
    command: StoredMatchPredictionCommand,
) -> Result<MatchPredictionReadiness, String> {
    state
        .service
        .inspect_match_prediction_readiness(command)
        .await
        .map_err(|error| error.to_string())
}
