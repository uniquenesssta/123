use crate::bootstrap::AppState;
use football_domain::{EntityMatchRequest, EntityMatchResult};
use tauri::State;

#[tauri::command]
pub async fn resolve_entity_reference(
    state: State<'_, AppState>,
    request: EntityMatchRequest,
) -> Result<EntityMatchResult, String> {
    state
        .service
        .resolve_entity_reference(request)
        .await
        .map_err(|error| error.to_string())
}
