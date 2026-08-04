use super::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn read_workspace_state(state: State<'_, AppState>) -> Result<Value, String> {
    state.workspace_state.read()
}

#[tauri::command]
pub fn save_workspace_state(state: State<'_, AppState>, document: Value) -> Result<(), String> {
    state.workspace_state.write(&document)
}

#[tauri::command]
pub fn clear_workspace_state(state: State<'_, AppState>) -> Result<Value, String> {
    state.workspace_state.clear()
}
