use super::{coordinator, response::BootstrapResponse};
use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn bootstrap(state: State<'_, AppState>) -> Result<BootstrapResponse, String> {
    coordinator::execute(&state).await
}
