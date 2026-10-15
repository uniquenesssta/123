use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub fn clear_issue_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.issue_log.clear()
}
