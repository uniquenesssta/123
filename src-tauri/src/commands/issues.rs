use super::AppState;
use crate::issue_log::{IssueLogDraft, IssueLogEntry, IssueLogStore};
use football_domain::JobStatus;
use tauri::State;

pub(super) fn record_failed_background_jobs(
    issue_log: &IssueLogStore,
    jobs: &[football_domain::BackgroundJob],
) {
    for job in jobs.iter().filter(|job| job.status == JobStatus::Failed) {
        if let Some(message) = job.error_message.as_ref() {
            let _ = issue_log.record(IssueLogDraft {
                severity: "error".to_string(),
                source: "background_job".to_string(),
                operation: format!("后台任务：{}", job.job_type),
                user_message: "后台分析任务执行失败".to_string(),
                technical_message: message.clone(),
                occurrence_key: Some(format!("{}:{}", job.id, job.attempts)),
            });
        }
    }
}

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
#[tauri::command]
pub fn clear_issue_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.issue_log.clear()
}
#[tauri::command]
pub fn export_issue_logs(state: State<'_, AppState>, output_path: String) -> Result<(), String> {
    state
        .issue_log
        .export_text(std::path::Path::new(&output_path))
}
