use crate::bootstrap::AppState;
use football_application::{PredictionExecution, StoredMatchPredictionCommand};
use tauri::State;

#[tauri::command]
pub async fn execute_shadow_prediction_from_match(
    state: State<'_, AppState>,
    command: StoredMatchPredictionCommand,
) -> Result<PredictionExecution, String> {
    state
        .service
        .execute_shadow_prediction_from_match(command)
        .await
        .map_err(|error| error.to_string())
}
