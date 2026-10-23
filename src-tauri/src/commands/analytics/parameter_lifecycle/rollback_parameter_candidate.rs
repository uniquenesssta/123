use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn rollback_parameter_candidate(
    state: State<'_, AppState>,
    request: football_domain::ParameterRollbackRequest,
) -> Result<football_domain::ParameterPromotionDecisionRecord, String> {
    state
        .service
        .rollback_parameter_candidate(request)
        .await
        .map_err(|error| error.to_string())
}
