use super::coordinator;
use crate::bootstrap::AppState;
use football_persistence_postgres::DatabaseHealth;
use tauri::State;

#[tauri::command]
pub async fn reset_database(
    state: State<'_, AppState>,
    confirmation: String,
) -> Result<DatabaseHealth, String> {
    coordinator::execute(&state, confirmation).await
}
