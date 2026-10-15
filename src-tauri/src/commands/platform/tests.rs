use super::background_jobs::record_failed_background_jobs;
use crate::issue_log::IssueLogStore;
use football_domain::{BackgroundJob, JobStatus};
use serde_json::json;

fn job(status: JobStatus, error: Option<&str>, attempts: i32) -> BackgroundJob {
    serde_json::from_value(json!({
        "id": "00000000-0000-0000-0000-000000000001", "job_type": "analysis_job",
        "status": status, "progress": 0.0, "payload": {}, "result": null,
        "error_message": error, "priority": 0, "attempts": attempts, "max_attempts": 3,
        "cancellation_requested": false, "available_at": "2026-01-01T00:00:00Z",
        "created_at": "2026-01-01T00:00:00Z", "started_at": null, "finished_at": null,
        "updated_at": "2026-01-01T00:00:00Z",
    }))
    .expect("original job DTO")
}

#[test]
fn background_scan_keeps_failed_only_filter_and_original_issue_projection() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = IssueLogStore::new(directory.path().join("issues.json"));
    let mut jobs: Vec<_> = [
        JobStatus::Queued,
        JobStatus::Running,
        JobStatus::Succeeded,
        JobStatus::Cancelled,
    ]
    .into_iter()
    .map(|status| job(status, Some("ignored failure text"), 1))
    .collect();
    jobs.push(job(JobStatus::Failed, None, 1));
    jobs.push(job(JobStatus::Failed, Some("original failure"), 2));
    record_failed_background_jobs(&store, &jobs);
    let entries = store.list(500).expect("issues");
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.source, "background_job");
    assert_eq!(entry.severity, "error");
    assert_eq!(entry.operations, ["后台任务：analysis_job"]);
    assert_eq!(entry.user_message, "后台分析任务执行失败");
    assert_eq!(entry.technical_message, "original failure");
    assert_eq!(
        entry.occurrence_keys,
        ["00000000-0000-0000-0000-000000000001:2"]
    );
}

#[test]
fn repeated_scan_deduplicates_same_attempt_but_records_a_new_attempt() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let store = IssueLogStore::new(directory.path().join("issues.json"));
    record_failed_background_jobs(&store, &[job(JobStatus::Failed, Some("same failure"), 1)]);
    record_failed_background_jobs(&store, &[job(JobStatus::Failed, Some("same failure"), 1)]);
    assert_eq!(
        store.list(500).expect("same attempt")[0].occurrence_count,
        1
    );
    record_failed_background_jobs(&store, &[job(JobStatus::Failed, Some("same failure"), 2)]);
    let entries = store.list(500).expect("next attempt");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].occurrence_count, 2);
    assert_eq!(entries[0].occurrence_keys.len(), 2);
}
