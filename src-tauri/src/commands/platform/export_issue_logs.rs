use crate::bootstrap::AppState;
use crate::issue_log::IssueLogEntry;
use tauri::State;

#[tauri::command]
pub fn export_issue_logs(
    state: State<'_, AppState>,
    output_path: String,
    expected_entries: Option<Vec<IssueLogEntry>>,
) -> Result<(), String> {
    match expected_entries {
        Some(entries) => state
            .issue_log
            .export_snapshot(std::path::Path::new(&output_path), Some(&entries)),
        None => state
            .issue_log
            .export_text(std::path::Path::new(&output_path)),
    }
}
