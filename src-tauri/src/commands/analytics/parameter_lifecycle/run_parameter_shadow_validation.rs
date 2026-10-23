use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn run_parameter_shadow_validation(
    state: State<'_, AppState>,
    request: football_domain::ParameterShadowValidationRequest,
) -> Result<football_domain::ParameterShadowValidationRecord, String> {
    state
        .service
        .run_parameter_shadow_validation(request)
        .await
        .map_err(|error| error.to_string())
}
