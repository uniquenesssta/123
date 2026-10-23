use crate::bootstrap::AppState;
use football_application::{PredictionCommand, PredictionExecution};
use tauri::State;

#[tauri::command]
pub async fn execute_prediction(
    state: State<'_, AppState>,
    command: PredictionCommand,
) -> Result<PredictionExecution, String> {
    state
        .service
        .execute_prediction(command)
        .await
        .map_err(|error| error.to_string())
}
