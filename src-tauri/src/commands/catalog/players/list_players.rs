use crate::bootstrap::AppState;
use football_domain::{PlayerListPage, PlayerListQuery};
use tauri::State;

#[tauri::command]
pub async fn list_players(
    state: State<'_, AppState>,
    query: PlayerListQuery,
) -> Result<PlayerListPage, String> {
    state
        .service
        .list_players(query)
        .await
        .map_err(|error| error.to_string())
}
