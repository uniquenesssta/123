use crate::issue_log::{IssueLogDraft, IssueLogStore};
use football_domain::JobStatus;

pub(in crate::commands) fn record_failed_background_jobs(
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
