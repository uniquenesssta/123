use crate::bootstrap::AppState;
use football_domain::{CoachListItem, CoachListQuery};
use tauri::State;

#[tauri::command]
pub async fn list_coaches(
    state: State<'_, AppState>,
    query: CoachListQuery,
) -> Result<Vec<CoachListItem>, String> {
    state
        .service
        .list_coaches(query)
        .await
        .map_err(|error| error.to_string())
}
