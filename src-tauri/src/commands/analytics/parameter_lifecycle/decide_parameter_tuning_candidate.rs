use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn decide_parameter_tuning_candidate(
    state: State<'_, AppState>,
    draft: football_domain::ParameterTuningDecisionDraft,
) -> Result<football_domain::ParameterTuningCandidateRecord, String> {
    state
        .service
        .decide_parameter_tuning_candidate(draft)
        .await
        .map_err(|error| error.to_string())
}
