use crate::bootstrap::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn save_workspace_state(state: State<'_, AppState>, document: Value) -> Result<(), String> {
    state.workspace_state.write(&document)
}
