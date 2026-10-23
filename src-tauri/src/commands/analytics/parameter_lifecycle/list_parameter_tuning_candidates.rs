use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_parameter_tuning_candidates(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::ParameterTuningCandidateRecord>, String> {
    state
        .service
        .list_parameter_tuning_candidates(limit)
        .await
        .map_err(|error| error.to_string())
}
