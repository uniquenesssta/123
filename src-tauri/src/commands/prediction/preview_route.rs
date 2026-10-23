use crate::bootstrap::AppState;
use football_application::RoutePreviewCommand;
use football_domain::RouteDecision;
use tauri::State;

#[tauri::command]
pub async fn preview_route(
    state: State<'_, AppState>,
    command: RoutePreviewCommand,
) -> Result<RouteDecision, String> {
    state
        .service
        .preview_route(command)
        .await
        .map_err(|error| error.to_string())
}
