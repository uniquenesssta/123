use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn hide_model_run_history(
    state: State<'_, AppState>,
    run_id: String,
    reason: Option<String>,
) -> Result<(), String> {
    let parsed = parse_uuid(&run_id, "运行 ID")?;
    state
        .service
        .hide_run_from_history(parsed, reason)
        .await
        .map_err(|error| error.to_string())
}
