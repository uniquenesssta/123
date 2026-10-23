use crate::bootstrap::AppState;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn dry_run_default_fixture(state: State<'_, AppState>) -> Result<Value, String> {
    let output = state
        .service
        .dry_run_default_fixture()
        .map_err(|error| error.to_string())?;
    serde_json::to_value(output).map_err(|error| error.to_string())
}
