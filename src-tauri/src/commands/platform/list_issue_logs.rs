use super::background_jobs::record_failed_background_jobs;
use crate::bootstrap::AppState;
use crate::issue_log::{IssueLogDraft, IssueLogEntry};
use tauri::State;

#[tauri::command]
pub async fn list_issue_logs(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<IssueLogEntry>, String> {
    if state.service.is_database_connected().await {
        match state.service.list_background_jobs(500).await {
            Ok(jobs) => record_failed_background_jobs(&state.issue_log, &jobs),
            Err(error) => {
                let message = error.to_string();
                let _ = state.issue_log.record(IssueLogDraft {
                    severity: "error".to_string(),
                    source: "backend".to_string(),
                    operation: "读取后台任务问题".to_string(),
                    user_message: "读取后台任务状态失败".to_string(),
                    technical_message: message.clone(),
                    occurrence_key: Some(format!("background-job-scan:{message}")),
                });
            }
        }
    }
    state.issue_log.list(limit.unwrap_or(500) as usize)
}
