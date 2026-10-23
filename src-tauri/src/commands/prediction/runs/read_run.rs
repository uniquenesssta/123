use crate::{bootstrap::AppState, commands::parse_uuid};
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub async fn read_run(state: State<'_, AppState>, run_id: String) -> Result<Value, String> {
    let parsed = parse_uuid(&run_id, "运行 ID")?;
    state
        .service
        .read_run(parsed)
        .await
        .map_err(|error| error.to_string())
}
