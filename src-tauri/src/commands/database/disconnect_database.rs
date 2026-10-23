use crate::{bootstrap::AppState, config::DesktopConfig};
use tauri::State;

#[tauri::command]
pub async fn disconnect_database(state: State<'_, AppState>) -> Result<(), String> {
    execute(&state).await
}

pub(super) async fn execute(state: &AppState) -> Result<(), String> {
    DesktopConfig::default().save(&state.config_path)?;
    state.service.disconnect_database().await;
    Ok(())
}
