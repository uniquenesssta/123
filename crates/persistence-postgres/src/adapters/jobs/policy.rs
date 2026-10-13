use crate::{PersistenceError, PersistenceResult};
use football_domain::JobStatus;

pub(super) fn validate_job_type(job_type: &str) -> PersistenceResult<()> {
    if matches!(
        job_type,
        "refresh_analytics"
            | "data_quality_scan"
            | "query_performance_scan"
            | "full_analysis_refresh"
            | "p4_horizon_research"
            | "p4_horizon_freeze"
    ) {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(format!(
            "不支持的任务类型：{job_type}"
        )))
    }
}

pub(super) fn job_status(status: &str) -> PersistenceResult<JobStatus> {
    Ok(match status {
        "queued" => JobStatus::Queued,
        "running" => JobStatus::Running,
        "succeeded" => JobStatus::Succeeded,
        "failed" => JobStatus::Failed,
        "cancelled" => JobStatus::Cancelled,
        other => {
            return Err(PersistenceError::InvalidState(format!(
                "未知任务状态：{other}"
            )))
        }
    })
}

pub(super) fn failure_disposition(
    attempts: i32,
    max_attempts: i32,
    cancellation_requested: bool,
) -> (&'static str, bool) {
    if cancellation_requested {
        ("cancelled", true)
    } else if attempts < max_attempts {
        ("queued", false)
    } else {
        ("failed", true)
    }
}
