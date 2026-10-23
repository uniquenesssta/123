use crate::bootstrap::AppState;
use football_persistence_postgres::ModelRunListItem;
use tauri::State;

#[tauri::command]
pub async fn list_recent_runs(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<ModelRunListItem>, String> {
    state
        .service
        .list_recent_runs(limit)
        .await
        .map_err(|error| error.to_string())
}
