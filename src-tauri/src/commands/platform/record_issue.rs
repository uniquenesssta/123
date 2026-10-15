use crate::bootstrap::AppState;
use crate::issue_log::{IssueLogDraft, IssueLogEntry};
use tauri::State;

#[tauri::command]
pub fn record_issue(
    state: State<'_, AppState>,
    draft: IssueLogDraft,
) -> Result<IssueLogEntry, String> {
    let _ = state.runtime_log.record(
        &draft.severity,
        "issue_log",
        "issue_recorded",
        draft.occurrence_key.as_deref(),
        serde_json::json!({
            "source": &draft.source,
            "operation": &draft.operation,
            "user_message": &draft.user_message,
            "technical_message": &draft.technical_message,
        }),
    );
    state.issue_log.record(draft)
}
