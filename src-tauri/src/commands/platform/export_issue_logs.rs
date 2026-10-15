use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub fn export_issue_logs(state: State<'_, AppState>, output_path: String) -> Result<(), String> {
    state
        .issue_log
        .export_text(std::path::Path::new(&output_path))
}
