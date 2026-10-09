use super::{failure::mark_terminal_failure, process_next::settle_job};
use crate::ports::prediction::{P4OrchestrationQueuePort, SerializedP4OrchestrationResult};
use crate::ports::PortResult;
use crate::services::p4_orchestration::tests::task_fixture;
use crate::use_cases::prediction::tests::Probe;
use crate::ApplicationError;
use async_trait::async_trait;
use chrono::Utc;
use football_domain::{BackgroundJob, JobStatus, P4FreezeTaskState};
use serde_json::json;
use uuid::Uuid;

fn job_fixture(attempts: i32) -> BackgroundJob {
    let now = Utc::now();
    BackgroundJob {
        id: Uuid::from_u128(70),
        job_type: "p4_horizon_research".into(),
        status: JobStatus::Running,
        progress: 0.0,
        payload: json!({"task_id":Uuid::from_u128(30)}),
        result: None,
        error_message: None,
        priority: 10,
        attempts,
        max_attempts: 3,
        cancellation_requested: false,
        available_at: now,
        created_at: now,
        updated_at: now,
        started_at: Some(now),
        finished_at: None,
    }
}

#[async_trait]
impl P4OrchestrationQueuePort for Probe {
    async fn claim_next_p4_job(&self, _types: &[&str]) -> PortResult<Option<BackgroundJob>> {
        panic!("unexpected claim in settlement tests")
    }
    async fn complete_p4_job(
        &self,
        id: Uuid,
        result: &SerializedP4OrchestrationResult,
    ) -> PortResult<()> {
        self.call("complete_job")?;
        self.state
            .lock()
            .unwrap()
            .job_results
            .push((id, result.as_str().into()));
        Ok(())
    }
    async fn fail_p4_job(&self, id: Uuid, error: &str) -> PortResult<()> {
        self.call("fail_job")?;
        self.state
            .lock()
            .unwrap()
            .job_failures
            .push((id, error.into()));
        Ok(())
    }
}

#[tokio::test]
async fn successful_dispatch_serializes_once_and_completes_only_the_claimed_job() {
    let probe = Probe::new();
    let job = job_fixture(1);
    let result = json!({"task":"complete"});
    assert_eq!(
        settle_job(&probe, &job, Ok(result.clone())).await.unwrap(),
        Some(result.clone())
    );
    let state = probe.state.lock().unwrap();
    assert_eq!(state.calls, ["complete_job"]);
    assert_eq!(
        state.job_results,
        [(job.id, serde_json::to_string(&result).unwrap())]
    );
    assert!(state.job_failures.is_empty());
}

#[tokio::test]
async fn complete_failure_propagates_without_marking_task_or_failing_job() {
    let probe = Probe::new();
    probe.fail_at(Some("complete_job"));
    assert!(matches!(
        settle_job(&probe, &job_fixture(3), Ok(json!({}))).await,
        Err(ApplicationError::Port(_))
    ));
    assert_eq!(probe.calls(), ["complete_job"]);
    assert!(probe.state.lock().unwrap().job_failures.is_empty());
}

#[tokio::test]
async fn retryable_dispatch_failure_keeps_task_and_original_error() {
    let probe = Probe::new();
    let job = job_fixture(2);
    let error = settle_job(
        &probe,
        &job,
        Err(ApplicationError::Validation("original".into())),
    )
    .await
    .unwrap_err();
    assert!(matches!(error,ApplicationError::Validation(ref message) if message=="original"));
    let state = probe.state.lock().unwrap();
    assert_eq!(state.calls, ["fail_job"]);
    assert_eq!(state.job_failures.len(), 1);
    assert_eq!(state.job_failures[0].0, job.id);
}

#[tokio::test]
async fn exhausted_dispatch_failure_marks_legal_task_then_fails_queue() {
    let probe = Probe::new();
    let job = job_fixture(3);
    probe.state.lock().unwrap().task = Some(task_fixture(P4FreezeTaskState::ResearchRunning));
    assert!(settle_job(
        &probe,
        &job,
        Err(ApplicationError::Validation("original".into()))
    )
    .await
    .is_err());
    let state = probe.state.lock().unwrap();
    assert_eq!(state.calls, ["read_task", "transition", "fail_job"]);
    let transition = &state.transitions[0];
    assert_eq!(
        transition.expected_state,
        P4FreezeTaskState::ResearchRunning
    );
    assert_eq!(transition.next_state, P4FreezeTaskState::Failed);
    assert_eq!(transition.task_id, Uuid::from_u128(30));
    assert_eq!(
        transition.payload,
        json!({"job_id":job.id,"attempts":3,"max_attempts":3})
    );
    assert_eq!(transition.blockers, json!([state.job_failures[0].1]));
    assert_eq!(
        (
            transition.research_run_id,
            transition.research_job_id,
            transition.freeze_job_id,
            transition.snapshot_id
        ),
        (None, None, None, None)
    );
}

#[tokio::test]
async fn exhausted_failure_does_not_reopen_terminal_or_unreadable_task() {
    for state in P4FreezeTaskState::ALL
        .into_iter()
        .filter(|s| s.is_terminal())
    {
        let probe = Probe::new();
        probe.state.lock().unwrap().task = Some(task_fixture(state));
        mark_terminal_failure(&probe, &job_fixture(3), "failed").await;
        assert_eq!(probe.calls(), ["read_task"]);
        assert!(probe.state.lock().unwrap().transitions.is_empty());
    }
    let probe = Probe::new();
    let mut malformed = job_fixture(3);
    malformed.payload = json!({"task_id":"not-a-uuid"});
    mark_terminal_failure(&probe, &malformed, "failed").await;
    assert!(probe.calls().is_empty());
    probe.fail_at(Some("read_task"));
    mark_terminal_failure(&probe, &job_fixture(3), "failed").await;
    assert_eq!(probe.calls(), ["read_task"]);
}

#[tokio::test]
async fn task_terminal_transition_is_best_effort_but_queue_failure_propagates() {
    let probe = Probe::new();
    probe.state.lock().unwrap().task = Some(task_fixture(P4FreezeTaskState::Planned));
    probe.fail_at(Some("transition"));
    let error = settle_job(
        &probe,
        &job_fixture(3),
        Err(ApplicationError::Validation("original".into())),
    )
    .await
    .unwrap_err();
    assert!(matches!(error,ApplicationError::Validation(ref message) if message=="original"));
    assert_eq!(probe.calls(), ["read_task", "transition", "fail_job"]);
    assert_eq!(
        probe.state.lock().unwrap().task.as_ref().unwrap().state,
        P4FreezeTaskState::Planned
    );
    let probe = Probe::new();
    probe.fail_at(Some("fail_job"));
    let error = settle_job(
        &probe,
        &job_fixture(1),
        Err(ApplicationError::Validation("original".into())),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(error,ApplicationError::Port(ref error) if error.message=="injected fail_job")
    );
}
