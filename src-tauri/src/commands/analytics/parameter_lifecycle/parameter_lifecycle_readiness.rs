use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn parameter_lifecycle_readiness(
    state: State<'_, AppState>,
    request: football_domain::ParameterLifecycleReadinessRequest,
) -> Result<football_domain::ParameterLifecycleReadiness, String> {
    state
        .service
        .parameter_lifecycle_readiness(request)
        .await
        .map_err(|error| error.to_string())
}
