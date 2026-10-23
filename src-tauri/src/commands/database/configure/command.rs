use super::coordinator;
use crate::bootstrap::AppState;
use football_persistence_postgres::{DatabaseHealth, DatabaseOptions};
use tauri::State;

#[tauri::command]
pub async fn configure_database(
    state: State<'_, AppState>,
    options: DatabaseOptions,
) -> Result<DatabaseHealth, String> {
    coordinator::execute(&state, options).await
}
