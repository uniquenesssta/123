use super::policy::{failure_disposition, job_status, validate_job_type};
use crate::PersistenceError;
use football_domain::JobStatus;

#[test]
fn supports_exact_original_analysis_and_p4_task_types() {
    for kind in [
        "refresh_analytics",
        "data_quality_scan",
        "query_performance_scan",
        "full_analysis_refresh",
        "p4_horizon_research",
        "p4_horizon_freeze",
    ] {
        validate_job_type(kind).unwrap();
    }
    for kind in [
        "",
        " refresh_analytics",
        "refresh_analytics ",
        "Refresh_analytics",
        "unknown",
    ] {
        assert!(
            matches!(validate_job_type(kind).unwrap_err(), PersistenceError::InvalidState(message) if message == format!("不支持的任务类型：{kind}"))
        );
    }
}

#[test]
fn decodes_all_five_original_job_states() {
    for (value, expected) in [
        ("queued", JobStatus::Queued),
        ("running", JobStatus::Running),
        ("succeeded", JobStatus::Succeeded),
        ("failed", JobStatus::Failed),
        ("cancelled", JobStatus::Cancelled),
    ] {
        assert_eq!(job_status(value).unwrap(), expected);
    }
}

#[test]
fn rejects_unknown_state_without_trimming_case_conversion_or_aliases() {
    for value in [
        "", " queued", "queued ", "Queued", "canceled", "success", "unknown",
    ] {
        assert!(
            matches!(job_status(value).unwrap_err(), PersistenceError::InvalidState(message) if message == format!("未知任务状态：{value}"))
        );
    }
}

#[test]
fn cancellation_has_priority_over_retry_budget_and_finishes_failure() {
    for (attempts, maximum) in [
        (0, 3),
        (3, 3),
        (4, 3),
        (i32::MIN, i32::MAX),
        (i32::MAX, i32::MIN),
    ] {
        assert_eq!(
            failure_disposition(attempts, maximum, true),
            ("cancelled", true)
        );
    }
}

#[test]
fn retries_strictly_below_original_attempt_budget() {
    for (attempts, maximum) in [(0, 1), (2, 3), (i32::MIN, 0), (i32::MAX - 1, i32::MAX)] {
        assert_eq!(
            failure_disposition(attempts, maximum, false),
            ("queued", false)
        );
    }
}

#[test]
fn reaches_terminal_failure_at_or_above_budget_without_new_validation() {
    for (attempts, maximum) in [(3, 3), (4, 3), (0, 0), (i32::MAX, i32::MAX), (0, i32::MIN)] {
        assert_eq!(
            failure_disposition(attempts, maximum, false),
            ("failed", true)
        );
    }
}
