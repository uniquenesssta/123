use crate::bootstrap::AppState;
use football_domain::{TeamListPage, TeamListQuery};
use tauri::State;

#[tauri::command]
pub async fn list_teams(
    state: State<'_, AppState>,
    query: TeamListQuery,
) -> Result<TeamListPage, String> {
    state
        .service
        .list_teams(query)
        .await
        .map_err(|error| error.to_string())
}
