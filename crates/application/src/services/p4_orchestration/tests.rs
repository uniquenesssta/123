use crate::use_cases::prediction::shared::p4_planning::{
    canonical_fact_keys, horizon_priority, is_p4_model,
};
use football_domain::P4Horizon;
use std::collections::BTreeSet;

#[test]
fn formal_fact_set_has_twenty_nine_unique_fields() {
    let fields = canonical_fact_keys();
    assert_eq!(fields.len(), 29);
    assert_eq!(fields.iter().collect::<BTreeSet<_>>().len(), 29);
}

#[test]
fn only_p4_models_enter_stage_f() {
    assert!(is_p4_model("p4"));
    assert!(is_p4_model("p4_knockout_90"));
    assert!(!is_p4_model("p7"));
}

#[test]
fn canonical_horizon_priority_increases_toward_kickoff() {
    assert!(horizon_priority(P4Horizon::T24h) < horizon_priority(P4Horizon::T6h));
    assert!(horizon_priority(P4Horizon::T6h) < horizon_priority(P4Horizon::T90m));
    assert!(horizon_priority(P4Horizon::T90m) < horizon_priority(P4Horizon::T1h));
}

use crate::model_registry::ModelRegistry;
use crate::ports::analytics::{AnalyticsJobProgressPayload, AnalyticsJobResult, JobQueuePort};
use crate::ports::prediction::{P4FreezeExecutionPort, PredictionWorkflowPort};
use crate::ports::research::ResearchArtifactPort;
use crate::ports::PortResult;
use crate::use_cases::prediction::{execute_p4_freeze, tests::Probe};
use crate::use_cases::research::p4_worker;
use crate::ApplicationError;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use football_domain::{
    BackgroundJob, CompetitionProfileVersionDraft, CompetitionProfileVersionRecord,
    EnqueueJobDraft, JobStatus, P4FreezeReadiness, P4FreezeTaskDraft, P4FreezeTaskEventRecord,
    P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition, P4MatchWorkspace,
    P4PlanningMatchContext, P4RoutedFact, P4TaskWorkspace, PrematchSnapshotBundle,
    PrematchSnapshotDraft, PrematchSnapshotRecord, PromptVersionDraft, PromptVersionRecord,
    ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft,
    SchemaVersionRecord, SourcePolicyVersionDraft, SourcePolicyVersionRecord,
};
use serde_json::json;
use uuid::Uuid;

fn task_fixture(state: P4FreezeTaskState) -> P4FreezeTaskRecord {
    let now = Utc::now();
    P4FreezeTaskRecord {
        id: Uuid::from_u128(30),
        match_id: Uuid::from_u128(31),
        match_key: "test-match".into(),
        horizon: P4Horizon::T1h,
        kickoff_at: now + Duration::hours(1),
        data_cutoff_at: now - Duration::hours(1),
        research_due_at: now - Duration::hours(2),
        freeze_deadline_at: now + Duration::hours(1),
        rule_package_id: Uuid::from_u128(10),
        model_version_id: Uuid::from_u128(11),
        parameter_set_id: Uuid::from_u128(12),
        competition_profile_id: Uuid::from_u128(13),
        research_schema_version_id: Uuid::from_u128(14),
        snapshot_schema_version_id: Uuid::from_u128(15),
        requested_fact_keys: vec!["fact".into()],
        trace_id: Uuid::from_u128(32),
        state,
        research_run_id: Some(Uuid::from_u128(33)),
        research_job_id: None,
        freeze_job_id: None,
        snapshot_id: None,
        blockers: json!([]),
        task_fingerprint: "test-fingerprint".into(),
        idempotency_key: "test-task-key".into(),
        metadata: json!({}),
        created_at: now,
        updated_at: now,
    }
}

fn workflow_probe(state: P4FreezeTaskState, ready: bool) -> Probe {
    let probe = Probe::new();
    {
        let mut value = probe.state.lock().unwrap();
        value.task = Some(task_fixture(state));
        value.readiness = Some(P4FreezeReadiness {
            task_id: Uuid::from_u128(30),
            ready,
            research_status: Some("SUCCEEDED".into()),
            requested_fact_count: 1,
            routed_fact_count: u32::from(ready),
            missing_fact_count: u32::from(!ready),
            ignored_fact_count: 0,
            blocked_fact_count: 0,
            blockers: if ready {
                vec![]
            } else {
                vec!["missing_fact".into()]
            },
        });
    }
    probe
}

async fn freeze(probe: &Probe) -> crate::ApplicationResult<serde_json::Value> {
    execute_p4_freeze::execute(
        probe,
        &ModelRegistry::new(),
        Uuid::from_u128(30),
        Uuid::from_u128(40),
    )
    .await
}

#[tokio::test]
async fn freeze_terminal_cancelled_and_failed_tasks_are_repeatable_read_only_noops() {
    for state in [
        P4FreezeTaskState::Cancelled,
        P4FreezeTaskState::Failed,
        P4FreezeTaskState::Frozen,
    ] {
        let probe = workflow_probe(state, true);
        for _ in 0..2 {
            let result = freeze(&probe).await.unwrap();
            assert_eq!(result["noop"], true);
            assert_eq!(result["state"], state.as_str());
        }
        assert_eq!(probe.calls(), ["read_task", "read_task"]);
        assert!(probe.state.lock().unwrap().transitions.is_empty());
    }
}

#[tokio::test]
async fn freeze_recovers_existing_snapshot_without_reexecuting_model_or_snapshot_write() {
    let probe = workflow_probe(P4FreezeTaskState::Freezing, true);
    probe.state.lock().unwrap().snapshot_id = Some(Uuid::from_u128(50));
    let result = freeze(&probe).await.unwrap();
    assert_eq!(result["recovered"], true);
    assert_eq!(result["snapshot_id"], Uuid::from_u128(50).to_string());
    assert_eq!(freeze(&probe).await.unwrap()["noop"], true);
    assert_eq!(
        probe.calls(),
        ["read_task", "find_snapshot", "transition", "read_task"]
    );
    let value = probe.state.lock().unwrap();
    let transition = &value.transitions[0];
    assert_eq!(value.transitions.len(), 1);
    assert_eq!(transition.expected_state, P4FreezeTaskState::Freezing);
    assert_eq!(transition.next_state, P4FreezeTaskState::Frozen);
    assert_eq!(transition.snapshot_id, Some(Uuid::from_u128(50)));
    assert_eq!(
        transition.payload["job_id"],
        Uuid::from_u128(40).to_string()
    );
}

#[tokio::test]
async fn existing_snapshot_in_wrong_state_is_rejected_without_transition() {
    let probe = workflow_probe(P4FreezeTaskState::ReadyToFreeze, true);
    probe.state.lock().unwrap().snapshot_id = Some(Uuid::from_u128(50));
    assert!(matches!(
        freeze(&probe).await,
        Err(ApplicationError::Validation(_))
    ));
    assert_eq!(probe.calls(), ["read_task", "find_snapshot"]);
    assert!(probe.state.lock().unwrap().transitions.is_empty());
}

#[tokio::test]
async fn freeze_early_claim_and_missed_deadline_do_not_prepare_input_or_run_model() {
    let early = workflow_probe(P4FreezeTaskState::ReadyToFreeze, true);
    early
        .state
        .lock()
        .unwrap()
        .task
        .as_mut()
        .unwrap()
        .data_cutoff_at = Utc::now() + Duration::hours(1);
    assert!(matches!(
        freeze(&early).await,
        Err(ApplicationError::Validation(_))
    ));
    assert_eq!(early.calls(), ["read_task", "find_snapshot"]);
    let late = workflow_probe(P4FreezeTaskState::ReadyToFreeze, true);
    late.state
        .lock()
        .unwrap()
        .task
        .as_mut()
        .unwrap()
        .freeze_deadline_at = Utc::now() - Duration::hours(1);
    assert_eq!(freeze(&late).await.unwrap()["state"], "MISSED");
    assert_eq!(freeze(&late).await.unwrap()["noop"], true);
    assert_eq!(
        late.calls(),
        ["read_task", "find_snapshot", "transition", "read_task"]
    );
    assert_eq!(
        late.state.lock().unwrap().transitions[0].next_state,
        P4FreezeTaskState::Missed
    );
}

#[tokio::test]
async fn freeze_failed_readiness_blocks_without_entering_freezing_or_writing_snapshot() {
    let probe = workflow_probe(P4FreezeTaskState::ReadyToFreeze, false);
    assert_eq!(freeze(&probe).await.unwrap()["state"], "BLOCKED");
    assert_eq!(
        probe.calls(),
        ["read_task", "find_snapshot", "readiness", "transition"]
    );
    let value = probe.state.lock().unwrap();
    let transition = &value.transitions[0];
    assert_eq!(transition.expected_state, P4FreezeTaskState::ReadyToFreeze);
    assert_eq!(transition.next_state, P4FreezeTaskState::Blocked);
    assert_eq!(transition.blockers, json!(["missing_fact"]));
    assert!(transition.snapshot_id.is_none());
}

#[tokio::test]
async fn freeze_port_failures_stop_later_calls_and_preserve_last_successful_state() {
    for (failure, expected, state) in [
        (
            "read_task",
            vec!["read_task"],
            P4FreezeTaskState::ReadyToFreeze,
        ),
        (
            "find_snapshot",
            vec!["read_task", "find_snapshot"],
            P4FreezeTaskState::ReadyToFreeze,
        ),
        (
            "readiness",
            vec!["read_task", "find_snapshot", "readiness"],
            P4FreezeTaskState::ReadyToFreeze,
        ),
        (
            "transition",
            vec!["read_task", "find_snapshot", "readiness", "transition"],
            P4FreezeTaskState::ReadyToFreeze,
        ),
        (
            "routed_facts",
            vec![
                "read_task",
                "find_snapshot",
                "readiness",
                "transition",
                "routed_facts",
            ],
            P4FreezeTaskState::Freezing,
        ),
    ] {
        let probe = workflow_probe(P4FreezeTaskState::ReadyToFreeze, true);
        probe.fail_at(Some(failure));
        assert!(matches!(
            freeze(&probe).await,
            Err(ApplicationError::Port(_))
        ));
        assert_eq!(probe.calls(), expected, "failure at {failure}");
        assert_eq!(
            probe.state.lock().unwrap().task.as_ref().unwrap().state,
            state
        );
        assert!(probe.state.lock().unwrap().snapshot_id.is_none());
    }
}

#[tokio::test]
async fn successful_research_enqueues_at_cutoff_before_registering_ready_state() {
    let probe = workflow_probe(P4FreezeTaskState::ResearchSucceeded, true);
    let task = probe.state.lock().unwrap().task.clone().unwrap();
    let result = p4_worker::finalize_successful_research(&probe, &probe, &task)
        .await
        .unwrap();
    assert_eq!(result.state, P4FreezeTaskState::ReadyToFreeze);
    assert_eq!(probe.calls(), ["readiness", "enqueue", "transition"]);
    let value = probe.state.lock().unwrap();
    let draft = &value.enqueues[0];
    assert_eq!(draft.job_type, "p4_horizon_freeze");
    assert_eq!(draft.payload, json!({"task_id": task.id}));
    assert_eq!(
        draft.idempotency_key.as_deref(),
        Some(format!("p4-freeze-job:{}", task.id).as_str())
    );
    assert_eq!(draft.available_at, Some(task.data_cutoff_at));
    assert_eq!(draft.max_attempts, 3);
    assert_eq!(draft.priority, horizon_priority(task.horizon) + 100);
    assert_eq!(
        value.transitions[0].expected_state,
        P4FreezeTaskState::ResearchSucceeded
    );
    assert_eq!(value.transitions[0].freeze_job_id, result.freeze_job_id);
    assert_eq!(result.freeze_job_id, Some(Uuid::from_u128(60)));
}

#[tokio::test]
async fn research_readiness_failure_and_enqueue_failure_never_register_ready_state() {
    let blocked = workflow_probe(P4FreezeTaskState::ResearchSucceeded, false);
    let task = blocked.state.lock().unwrap().task.clone().unwrap();
    assert_eq!(
        p4_worker::finalize_successful_research(&blocked, &blocked, &task)
            .await
            .unwrap()
            .state,
        P4FreezeTaskState::Blocked
    );
    assert_eq!(blocked.calls(), ["readiness", "transition"]);
    assert_eq!(
        blocked.state.lock().unwrap().transitions[0].blockers,
        json!(["missing_fact"])
    );
    let failed = workflow_probe(P4FreezeTaskState::ResearchSucceeded, true);
    failed.fail_at(Some("enqueue"));
    let task = failed.state.lock().unwrap().task.clone().unwrap();
    assert!(matches!(
        p4_worker::finalize_successful_research(&failed, &failed, &task).await,
        Err(ApplicationError::Port(_))
    ));
    assert_eq!(failed.calls(), ["readiness", "enqueue"]);
    assert!(failed.state.lock().unwrap().transitions.is_empty());
    assert_eq!(
        failed.state.lock().unwrap().task.as_ref().unwrap().state,
        P4FreezeTaskState::ResearchSucceeded
    );
}

#[tokio::test]
async fn research_state_write_failure_retries_same_job_identity_without_duplicate_job() {
    let probe = workflow_probe(P4FreezeTaskState::ResearchSucceeded, true);
    probe.fail_at(Some("transition"));
    let task = probe.state.lock().unwrap().task.clone().unwrap();
    assert!(matches!(
        p4_worker::finalize_successful_research(&probe, &probe, &task).await,
        Err(ApplicationError::Port(_))
    ));
    assert_eq!(
        probe.state.lock().unwrap().task.as_ref().unwrap().state,
        P4FreezeTaskState::ResearchSucceeded
    );
    probe.fail_at(None);
    let result = p4_worker::finalize_successful_research(&probe, &probe, &task)
        .await
        .unwrap();
    assert_eq!(
        probe.calls(),
        [
            "readiness",
            "enqueue",
            "transition",
            "readiness",
            "enqueue",
            "transition"
        ]
    );
    let value = probe.state.lock().unwrap();
    assert_eq!(value.enqueues.len(), 2);
    assert_eq!(
        value.enqueues[0].idempotency_key,
        value.enqueues[1].idempotency_key
    );
    assert_eq!(value.queued_job_id, result.freeze_job_id);
    assert_eq!(value.transitions.len(), 1);
    assert_eq!(result.state, P4FreezeTaskState::ReadyToFreeze);
}

#[tokio::test]
async fn failed_snapshot_recovery_transition_can_retry_without_rewriting_snapshot() {
    let probe = workflow_probe(P4FreezeTaskState::Freezing, true);
    probe.state.lock().unwrap().snapshot_id = Some(Uuid::from_u128(50));
    probe.fail_at(Some("transition"));
    assert!(matches!(
        freeze(&probe).await,
        Err(ApplicationError::Port(_))
    ));
    assert_eq!(
        probe.state.lock().unwrap().task.as_ref().unwrap().state,
        P4FreezeTaskState::Freezing
    );
    probe.fail_at(None);
    assert_eq!(freeze(&probe).await.unwrap()["recovered"], true);
    assert_eq!(
        probe.calls(),
        [
            "read_task",
            "find_snapshot",
            "transition",
            "read_task",
            "find_snapshot",
            "transition"
        ]
    );
    let value = probe.state.lock().unwrap();
    assert_eq!(value.transitions.len(), 1);
    assert_eq!(
        value.task.as_ref().unwrap().snapshot_id,
        Some(Uuid::from_u128(50))
    );
}

#[tokio::test]
async fn research_readiness_port_error_cannot_enqueue_or_transition_task() {
    let probe = workflow_probe(P4FreezeTaskState::ResearchSucceeded, true);
    let task = probe.state.lock().unwrap().task.clone().unwrap();
    probe.fail_at(Some("readiness"));
    assert!(matches!(
        p4_worker::finalize_successful_research(&probe, &probe, &task).await,
        Err(ApplicationError::Port(_))
    ));
    assert_eq!(probe.calls(), ["readiness"]);
    assert!(probe.state.lock().unwrap().transitions.is_empty());
    assert!(probe.state.lock().unwrap().enqueues.is_empty());
}

#[async_trait]
impl PredictionWorkflowPort for Probe {
    async fn planning_match_context(&self, _match_id: Uuid) -> PortResult<P4PlanningMatchContext> {
        panic!("forbidden Port call: planning_match_context")
    }
    async fn find_freeze_task_by_idempotency(
        &self,
        _idempotency_key: &str,
    ) -> PortResult<Option<P4FreezeTaskRecord>> {
        panic!("forbidden Port call: find_freeze_task_by_idempotency")
    }
    async fn list_freeze_tasks(
        &self,
        _match_id: Option<Uuid>,
        _limit: u32,
    ) -> PortResult<Vec<P4FreezeTaskRecord>> {
        panic!("forbidden Port call: list_freeze_tasks")
    }
    async fn read_match_workspace(&self, _match_id: Uuid) -> PortResult<P4MatchWorkspace> {
        panic!("forbidden Port call: read_match_workspace")
    }
    async fn read_task_workspace(&self, _task_id: Uuid) -> PortResult<P4TaskWorkspace> {
        panic!("forbidden Port call: read_task_workspace")
    }
    async fn create_freeze_task(
        &self,
        _draft: &P4FreezeTaskDraft,
    ) -> PortResult<P4FreezeTaskRecord> {
        panic!("forbidden Port call: create_freeze_task")
    }
    async fn read_freeze_task(&self, task_id: Uuid) -> PortResult<P4FreezeTaskRecord> {
        self.call("read_task")?;
        let task = self.state.lock().unwrap().task.clone().unwrap();
        assert_eq!(task_id, task.id);
        Ok(task)
    }
    async fn list_freeze_task_events(
        &self,
        _task_id: Uuid,
    ) -> PortResult<Vec<P4FreezeTaskEventRecord>> {
        panic!("forbidden Port call: list_freeze_task_events")
    }
    async fn transition_freeze_task(
        &self,
        task_id: Uuid,
        transition: &P4FreezeTaskTransition,
    ) -> PortResult<P4FreezeTaskRecord> {
        self.call("transition")?;
        let mut value = self.state.lock().unwrap();
        let task = value.task.as_mut().unwrap();
        assert_eq!(task_id, task.id);
        assert_eq!(transition.task_id, task.id);
        assert_eq!(task.state, transition.expected_state);
        assert!(task.state.can_transition_to(transition.next_state));
        task.state = transition.next_state;
        if let Some(id) = transition.snapshot_id {
            task.snapshot_id = Some(id);
        }
        if let Some(id) = transition.freeze_job_id {
            task.freeze_job_id = Some(id);
        }
        task.blockers = transition.blockers.clone();
        let result = task.clone();
        value.transitions.push(transition.clone());
        Ok(result)
    }
    async fn freeze_readiness(&self, task_id: Uuid) -> PortResult<P4FreezeReadiness> {
        self.call("readiness")?;
        let ready = self.state.lock().unwrap().readiness.clone().unwrap();
        assert_eq!(task_id, ready.task_id);
        Ok(ready)
    }
}

#[async_trait]
impl JobQueuePort for Probe {
    async fn enqueue(&self, draft: &EnqueueJobDraft) -> PortResult<BackgroundJob> {
        self.call("enqueue")?;
        let mut value = self.state.lock().unwrap();
        if let Some(first) = value.enqueues.first() {
            assert_eq!(draft.idempotency_key, first.idempotency_key);
        }
        value.enqueues.push(draft.clone());
        let id = *value.queued_job_id.get_or_insert(Uuid::from_u128(60));
        let now = Utc::now();
        Ok(BackgroundJob {
            id,
            job_type: draft.job_type.clone(),
            status: JobStatus::Queued,
            progress: 0.0,
            payload: draft.payload.clone(),
            result: None,
            error_message: None,
            priority: draft.priority,
            attempts: 0,
            max_attempts: draft.max_attempts,
            cancellation_requested: false,
            available_at: draft.available_at.unwrap(),
            created_at: now,
            started_at: None,
            finished_at: None,
            updated_at: now,
        })
    }
    async fn list_jobs(&self, _limit: u32) -> PortResult<Vec<BackgroundJob>> {
        panic!("forbidden Port call: list_jobs")
    }
    async fn request_cancellation(&self, _job_id: Uuid) -> PortResult<BackgroundJob> {
        panic!("forbidden Port call: request_cancellation")
    }
    async fn retry(&self, _job_id: Uuid) -> PortResult<BackgroundJob> {
        panic!("forbidden Port call: retry")
    }
    async fn claim_next_by_types(&self, _job_types: &[&str]) -> PortResult<Option<BackgroundJob>> {
        panic!("forbidden Port call: claim_next_by_types")
    }
    async fn update_progress(
        &self,
        _job_id: Uuid,
        _progress: f64,
        _message: &str,
        _payload: AnalyticsJobProgressPayload,
    ) -> PortResult<bool> {
        panic!("forbidden Port call: update_progress")
    }
    async fn complete(&self, _job_id: Uuid, _result: AnalyticsJobResult) -> PortResult<()> {
        panic!("forbidden Port call: complete")
    }
    async fn fail(&self, _job_id: Uuid, _error_message: &str) -> PortResult<()> {
        panic!("forbidden Port call: fail")
    }
}

#[async_trait]
impl P4FreezeExecutionPort for Probe {
    async fn find_frozen_snapshot_id(&self, task: &P4FreezeTaskRecord) -> PortResult<Option<Uuid>> {
        self.call("find_snapshot")?;
        assert_eq!(
            task.id,
            self.state.lock().unwrap().task.as_ref().unwrap().id
        );
        Ok(self.state.lock().unwrap().snapshot_id)
    }
    async fn routed_facts(&self, task_id: Uuid) -> PortResult<Vec<P4RoutedFact>> {
        self.call("routed_facts")?;
        let _ = task_id;
        panic!("unexpected snapshot/model execution in guard and recovery tests")
    }
    async fn freeze_snapshot(
        &self,
        _draft: &PrematchSnapshotDraft,
    ) -> PortResult<PrematchSnapshotRecord> {
        panic!("forbidden Port call: freeze_snapshot")
    }
    async fn read_snapshot(&self, _snapshot_id: Uuid) -> PortResult<PrematchSnapshotBundle> {
        panic!("forbidden Port call: read_snapshot")
    }
}

#[async_trait]
impl ResearchArtifactPort for Probe {
    async fn read_schema(
        &self,
        _schema_key: &str,
        _version: &str,
    ) -> PortResult<SchemaVersionRecord> {
        panic!("forbidden Port call: read_schema")
    }
    async fn register_schema(
        &self,
        _draft: &SchemaVersionDraft,
    ) -> PortResult<SchemaVersionRecord> {
        panic!("forbidden Port call: register_schema")
    }
    async fn register_prompt(
        &self,
        _draft: &PromptVersionDraft,
    ) -> PortResult<PromptVersionRecord> {
        panic!("forbidden Port call: register_prompt")
    }
    async fn register_source_policy(
        &self,
        _draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord> {
        panic!("forbidden Port call: register_source_policy")
    }
    async fn register_competition_profile(
        &self,
        _draft: &CompetitionProfileVersionDraft,
    ) -> PortResult<CompetitionProfileVersionRecord> {
        panic!("forbidden Port call: register_competition_profile")
    }
    async fn create_run(&self, _draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord> {
        panic!("forbidden Port call: create_run")
    }
    async fn read_run(&self, _run_id: Uuid) -> PortResult<ResearchRunRecord> {
        panic!("forbidden Port call: read_run")
    }
    async fn record_run_event(
        &self,
        _draft: &ResearchRunEventDraft,
    ) -> PortResult<ResearchRunRecord> {
        panic!("forbidden Port call: record_run_event")
    }
}
