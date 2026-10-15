use crate::bootstrap::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn clear_workspace_state(state: State<'_, AppState>) -> Result<Value, String> {
    state.workspace_state.clear()
}
