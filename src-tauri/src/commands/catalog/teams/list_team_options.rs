use crate::bootstrap::AppState;
use football_domain::TeamOption;
use tauri::State;

#[tauri::command]
pub async fn list_team_options(
    state: State<'_, AppState>,
    search: Option<String>,
    limit: u32,
) -> Result<Vec<TeamOption>, String> {
    state
        .service
        .list_team_options(search, limit)
        .await
        .map_err(|error| error.to_string())
}
