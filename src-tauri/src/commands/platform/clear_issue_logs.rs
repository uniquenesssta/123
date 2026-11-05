use crate::bootstrap::AppState;
use crate::issue_log::IssueLogEntry;
use tauri::State;

#[tauri::command]
pub fn clear_issue_logs(
    state: State<'_, AppState>,
    expected_entries: Option<Vec<IssueLogEntry>>,
) -> Result<(), String> {
    match expected_entries {
        Some(entries) => state.issue_log.clear_snapshot(Some(&entries)),
        None => state.issue_log.clear(),
    }
}
