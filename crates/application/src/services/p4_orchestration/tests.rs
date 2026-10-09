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

pub(crate) fn task_fixture(state: P4FreezeTaskState) -> P4FreezeTaskRecord {
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

// Provider-owned JSON here is only a boundary fixture, never a probability regression oracle.
fn freeze_execution_probe() -> std::sync::Arc<Probe> {
    use crate::built_in_artifacts::{P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION, P4_SNAPSHOT_SCHEMA_KEY};
    use crate::use_cases::prediction::tests::command_fixture;
    use football_domain::{
        CompetitionKind, MatchRecord, MatchStatus, PreparedMatchPredictionInput,
    };
    let probe = std::sync::Arc::new(workflow_probe(P4FreezeTaskState::ReadyToFreeze, true));
    let mut state = probe.state.lock().unwrap();
    let task = state.task.clone().unwrap();
    let mut input = command_fixture().match_input;
    input["match_id"] = json!(task.match_key);
    input["kickoff_time"] = json!(task.kickoff_at.to_rfc3339());
    input["feature_quality_score"] = json!(0.6);
    state.prepared_input = Some(PreparedMatchPredictionInput {
        match_record: MatchRecord {
            id: task.match_id,
            external_key: task.match_key.clone(),
            competition_id: Some(Uuid::from_u128(21)),
            competition_name: None,
            season_id: Some(Uuid::from_u128(22)),
            stage_id: Some(Uuid::from_u128(23)),
            round_id: None,
            home_team_id: Uuid::from_u128(1),
            home_team_name: "Home".into(),
            away_team_id: Uuid::from_u128(2),
            away_team_name: "Away".into(),
            kickoff_time: task.kickoff_at,
            status: MatchStatus::Scheduled,
            venue: None,
        },
        competition_kind: CompetitionKind::League,
        snapshot_type: task.horizon.as_str().into(),
        match_input: input,
        data_quality: json!({"cutoff_aware":true}),
    });
    state.freeze_routes = Some(vec![]);
    state.provider_payload = Some(json!({"matrices": {"provider_chain": {
        "outcome":{"a_win":0.4,"draw":0.3,"b_win":0.3},
        "scorelines":[{"score":"1-0","probability":0.4}],"formal":true
    }}}));
    state.schemas.push(SchemaVersionRecord {
        id: task.snapshot_schema_version_id,
        schema_key: P4_SNAPSHOT_SCHEMA_KEY.into(),
        version: P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION.into(),
        schema_kind: "snapshot".into(),
        content_sha256: "fixture".into(),
        created_at: Utc::now(),
    });
    drop(state);
    probe
}

async fn execute_freeze_probe(
    probe: &std::sync::Arc<Probe>,
) -> crate::ApplicationResult<serde_json::Value> {
    execute_p4_freeze::execute(
        probe.as_ref(),
        &crate::use_cases::prediction::tests::probe_registry(probe),
        Uuid::from_u128(30),
        Uuid::from_u128(40),
    )
    .await
}

#[tokio::test]
async fn freeze_execution_pins_snapshot_provenance_and_commits_before_frozen_transition() {
    let probe = freeze_execution_probe();
    let value = execute_freeze_probe(&probe).await.unwrap();
    assert_eq!(value["state"], "FROZEN");
    assert_eq!(value["snapshot_id"], json!(Uuid::from_u128(50)));
    assert_eq!(
        probe.calls(),
        [
            "read_task",
            "find_snapshot",
            "readiness",
            "transition",
            "routed_facts",
            "prepare_input",
            "scope",
            "route",
            "model_supports",
            "predict",
            "save_run",
            "schema",
            "freeze_snapshot",
            "transition"
        ]
    );
    let state = probe.state.lock().unwrap();
    assert_eq!(state.frozen_drafts.len(), 1);
    let draft = &state.frozen_drafts[0];
    let task = state.task.as_ref().unwrap();
    assert_eq!(draft.data_cutoff_at, task.data_cutoff_at);
    assert!(draft.frozen_at >= draft.data_cutoff_at && draft.frozen_at <= task.freeze_deadline_at);
    assert_eq!(draft.trace_id, task.trace_id);
    assert_eq!(draft.research_run_id, task.research_run_id);
    assert_eq!(draft.model_version_id, task.model_version_id);
    assert_eq!(draft.parameter_set_id, task.parameter_set_id);
    assert_eq!(draft.competition_profile_id, task.competition_profile_id);
    assert_eq!(draft.schema_version_id, task.snapshot_schema_version_id);
    assert_eq!(
        draft.idempotency_key,
        format!("p4-prematch-snapshot:{}", task.id)
    );
    assert_eq!(draft.source_kind, football_domain::SnapshotSourceKind::Real);
    assert!((draft.quality_score - 0.8).abs() < f64::EPSILON);
    assert_eq!(draft.features.len(), 31);
    assert_eq!(draft.features[29].value, json!({"cutoff_aware":true}));
    assert_eq!(
        draft.probabilities.len(),
        1,
        "provider topology is not forced to four chains"
    );
    assert_eq!(draft.probabilities[0].chain_key, "provider_chain");
    assert_eq!(
        draft.input_payload["p4_output"],
        state.provider_payload.clone().unwrap()
    );
    assert_eq!(
        draft.input_payload["match_input"]["p4_orchestration"]["task_id"],
        json!(task.id)
    );
    assert_eq!(draft.metadata["model_run_id"], json!(Uuid::from_u128(99)));
    assert_eq!(
        state.transitions.last().unwrap().snapshot_id,
        Some(Uuid::from_u128(50))
    );
    assert_eq!(
        state.route_requests[0].explicit_rule_package_id,
        Some(task.rule_package_id)
    );
}

#[tokio::test]
async fn freeze_execution_stops_at_each_late_port_failure_without_snapshot_or_frozen_state() {
    use crate::ports::PortErrorKind;
    for boundary in [
        "routed_facts",
        "prepare_input",
        "scope",
        "route",
        "save_run",
        "schema",
        "freeze_snapshot",
    ] {
        for kind in [
            PortErrorKind::InvalidState,
            PortErrorKind::NotFound,
            PortErrorKind::Conflict,
            PortErrorKind::Unavailable,
            PortErrorKind::Serialization,
            PortErrorKind::Infrastructure,
        ] {
            let probe = freeze_execution_probe();
            probe.fail_at(Some(boundary));
            probe.state.lock().unwrap().failure_kind = Some(kind);
            let error = execute_freeze_probe(&probe).await.unwrap_err();
            assert!(
                matches!(error, ApplicationError::Port(ref actual) if actual.kind == kind && actual.message == format!("injected {boundary}"))
            );
            assert_eq!(probe.calls().last(), Some(&boundary));
            let state = probe.state.lock().unwrap();
            assert!(state.frozen_drafts.is_empty());
            assert_eq!(
                state.task.as_ref().unwrap().state,
                P4FreezeTaskState::Freezing
            );
            assert!(state.task.as_ref().unwrap().snapshot_id.is_none());
            assert_eq!(state.transitions.len(), 1);
        }
    }
}

#[tokio::test]
async fn freeze_execution_rejects_route_schema_and_provider_output_drift_before_snapshot() {
    for drift in [
        "rule",
        "model",
        "parameter",
        "profile",
        "schema",
        "matrices",
        "outcome",
        "scorelines",
    ] {
        let probe = freeze_execution_probe();
        {
            let mut state = probe.state.lock().unwrap();
            let mut route = crate::use_cases::prediction::tests::route_fixture();
            match drift {
                "rule" => route.rule_package_id = Uuid::from_u128(999),
                "model" => route.model_version_id = Uuid::from_u128(999),
                "parameter" => route.parameter_set_id = Uuid::from_u128(999),
                "profile" => route.competition_profile_id = Uuid::from_u128(999),
                "schema" => state.schemas[0].id = Uuid::from_u128(999),
                "matrices" => state.provider_payload = Some(json!({"matrices":{}})),
                "outcome" => {
                    state.provider_payload.as_mut().unwrap()["matrices"]["provider_chain"]
                        ["outcome"]["a_win"] = json!(1.1)
                }
                "scorelines" => {
                    state.provider_payload.as_mut().unwrap()["matrices"]["provider_chain"]
                        ["scorelines"] = json!([])
                }
                _ => unreachable!(),
            }
            state.planning_route = Some(route);
        }
        assert!(matches!(
            execute_freeze_probe(&probe).await,
            Err(ApplicationError::Validation(_))
        ));
        let state = probe.state.lock().unwrap();
        assert!(state.frozen_drafts.is_empty());
        assert_eq!(
            state.task.as_ref().unwrap().state,
            P4FreezeTaskState::Freezing
        );
        assert!(!state.calls.contains(&"freeze_snapshot"));
    }
}

#[tokio::test]
async fn freeze_execution_recovers_committed_snapshot_after_frozen_transition_failure() {
    let probe = freeze_execution_probe();
    probe.fail_at(Some("transition"));
    probe.state.lock().unwrap().failure_call_number = Some(2);
    assert!(matches!(
        execute_freeze_probe(&probe).await,
        Err(ApplicationError::Port(_))
    ));
    {
        let state = probe.state.lock().unwrap();
        assert_eq!(state.frozen_drafts.len(), 1);
        assert_eq!(state.snapshot_id, Some(Uuid::from_u128(50)));
        assert_eq!(
            state.task.as_ref().unwrap().state,
            P4FreezeTaskState::Freezing
        );
        assert!(state.task.as_ref().unwrap().snapshot_id.is_none());
    }
    probe.fail_at(None);
    let before = probe.calls().len();
    assert_eq!(
        execute_freeze_probe(&probe).await.unwrap()["recovered"],
        true
    );
    assert_eq!(
        &probe.calls()[before..],
        ["read_task", "find_snapshot", "transition"]
    );
    let state = probe.state.lock().unwrap();
    assert_eq!(
        state.frozen_drafts.len(),
        1,
        "recovery never repeats model or snapshot writes"
    );
    assert_eq!(
        state.task.as_ref().unwrap().state,
        P4FreezeTaskState::Frozen
    );
    assert_eq!(state.task.as_ref().unwrap().snapshot_id, state.snapshot_id);
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
    async fn planning_match_context(&self, match_id: Uuid) -> PortResult<P4PlanningMatchContext> {
        self.call("context")?;
        let context = self
            .state
            .lock()
            .unwrap()
            .planning_context
            .clone()
            .expect("unexpected planning context");
        assert_eq!(context.match_id, match_id);
        Ok(context)
    }
    async fn find_freeze_task_by_idempotency(
        &self,
        idempotency_key: &str,
    ) -> PortResult<Option<P4FreezeTaskRecord>> {
        self.call("find_task")?;
        Ok(self
            .state
            .lock()
            .unwrap()
            .planned_tasks
            .iter()
            .find(|task| task.idempotency_key == idempotency_key)
            .cloned())
    }
    async fn list_freeze_tasks(
        &self,
        _match_id: Option<Uuid>,
        _limit: u32,
    ) -> PortResult<Vec<P4FreezeTaskRecord>> {
        panic!("forbidden Port call: list_freeze_tasks")
    }
    async fn read_match_workspace(&self, match_id: Uuid) -> PortResult<P4MatchWorkspace> {
        self.call("match_workspace")?;
        let mut state = self.state.lock().unwrap();
        state.match_workspace_requests.push(match_id);
        Ok(state
            .match_workspace
            .clone()
            .expect("selected match workspace"))
    }
    async fn read_task_workspace(&self, task_id: Uuid) -> PortResult<P4TaskWorkspace> {
        self.call("task_workspace")?;
        let mut state = self.state.lock().unwrap();
        state.task_workspace_requests.push(task_id);
        Ok(state
            .task_workspace
            .clone()
            .expect("selected task workspace"))
    }
    async fn create_freeze_task(
        &self,
        draft: &P4FreezeTaskDraft,
    ) -> PortResult<P4FreezeTaskRecord> {
        self.call("create_task")?;
        let mut state = self.state.lock().unwrap();
        let now = Utc::now();
        let mut value = serde_json::to_value(draft).unwrap();
        let object = value.as_object_mut().unwrap();
        object.insert(
            "id".into(),
            json!(Uuid::from_u128(200 + state.planned_tasks.len() as u128)),
        );
        object.insert("created_at".into(), json!(now));
        object.insert("updated_at".into(), json!(now));
        object.insert("task_fingerprint".into(), json!("test-fingerprint"));
        object.insert("blockers".into(), json!([]));
        let task: P4FreezeTaskRecord = serde_json::from_value(value).unwrap();
        state.task_drafts.push(draft.clone());
        state.planned_tasks.push(task.clone());
        Ok(task)
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
        let task = if value.task.as_ref().is_some_and(|task| task.id == task_id) {
            value.task.as_mut().unwrap()
        } else {
            value
                .planned_tasks
                .iter_mut()
                .find(|task| task.id == task_id)
                .unwrap()
        };
        assert_eq!(task_id, task.id);
        assert_eq!(transition.task_id, task.id);
        assert_eq!(task.state, transition.expected_state);
        assert!(task.state.can_transition_to(transition.next_state));
        task.state = transition.next_state;
        if let Some(id) = transition.research_run_id {
            task.research_run_id = Some(id);
        }
        if let Some(id) = transition.research_job_id {
            task.research_job_id = Some(id);
        }
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
        if let Some(first) = value
            .enqueues
            .first()
            .filter(|_| value.planning_context.is_none())
        {
            assert_eq!(draft.idempotency_key, first.idempotency_key);
        }
        value.enqueues.push(draft.clone());
        let id = if value.planning_context.is_some() {
            let next = Uuid::from_u128(100 + value.planned_job_ids.len() as u128);
            *value
                .planned_job_ids
                .entry(draft.idempotency_key.clone().unwrap())
                .or_insert(next)
        } else {
            *value.queued_job_id.get_or_insert(Uuid::from_u128(60))
        };
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
        let state = self.state.lock().unwrap();
        assert_eq!(state.task.as_ref().unwrap().id, task_id);
        Ok(state
            .freeze_routes
            .clone()
            .expect("unexpected snapshot/model execution in guard and recovery tests"))
    }
    async fn freeze_snapshot(
        &self,
        draft: &PrematchSnapshotDraft,
    ) -> PortResult<PrematchSnapshotRecord> {
        self.call("freeze_snapshot")?;
        let mut state = self.state.lock().unwrap();
        assert!(state.freeze_routes.is_some(), "unselected freeze_snapshot");
        let id = Uuid::from_u128(50);
        state.frozen_drafts.push(draft.clone());
        state.snapshot_id = Some(id);
        Ok(PrematchSnapshotRecord {
            id,
            match_id: draft.match_id,
            match_key: draft.match_key.clone(),
            horizon: draft.horizon,
            data_cutoff_at: draft.data_cutoff_at,
            frozen_at: draft.frozen_at,
            snapshot_fingerprint: "test-snapshot-fingerprint".into(),
            idempotency_key: draft.idempotency_key.clone(),
            source_kind: draft.source_kind,
            evidence_scope: draft.source_kind.evidence_scope().into(),
            created: true,
            created_at: Utc::now(),
        })
    }
    async fn read_snapshot(&self, _snapshot_id: Uuid) -> PortResult<PrematchSnapshotBundle> {
        panic!("forbidden Port call: read_snapshot")
    }
}

#[async_trait]
impl ResearchArtifactPort for Probe {
    async fn read_schema(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PortResult<SchemaVersionRecord> {
        self.call("schema")?;
        let mut state = self.state.lock().unwrap();
        state.schema_reads.push((schema_key.into(), version.into()));
        Ok(state
            .schemas
            .iter()
            .find(|schema| schema.schema_key == schema_key && schema.version == version)
            .cloned()
            .expect("unexpected schema version"))
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

fn planning_probe(kickoff_at: chrono::DateTime<Utc>) -> Probe {
    use crate::built_in_artifacts::{
        P4_RESEARCH_SCHEMA_ARTIFACT_VERSION, P4_RESEARCH_SCHEMA_KEY,
        P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION, P4_SNAPSHOT_SCHEMA_KEY,
    };
    let probe = Probe::new();
    let mut state = probe.state.lock().unwrap();
    state.planning_context = Some(P4PlanningMatchContext {
        match_id: Uuid::from_u128(31),
        match_key: "test-match".into(),
        kickoff_at,
        competition_id: Some(Uuid::from_u128(21)),
        season_id: Some(Uuid::from_u128(22)),
        stage_id: Some(Uuid::from_u128(23)),
        competition_kind: football_domain::CompetitionKind::League,
        home_team_name: "Home".into(),
        away_team_name: "Away".into(),
    });
    let mut route = crate::use_cases::prediction::tests::route_fixture();
    route.routing.supported_snapshot_types = P4Horizon::CANONICAL
        .into_iter()
        .map(|h| h.as_str().into())
        .collect();
    state.planning_route = Some(route);
    state.schemas = [
        (
            14,
            P4_RESEARCH_SCHEMA_KEY,
            P4_RESEARCH_SCHEMA_ARTIFACT_VERSION,
        ),
        (
            15,
            P4_SNAPSHOT_SCHEMA_KEY,
            P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION,
        ),
    ]
    .into_iter()
    .map(|(id, key, version)| SchemaVersionRecord {
        id: Uuid::from_u128(id),
        schema_key: key.into(),
        version: version.into(),
        schema_kind: "test".into(),
        content_sha256: "test".into(),
        created_at: Utc::now(),
    })
    .collect();
    drop(state);
    probe
}

fn planning_command() -> football_domain::PlanP4HorizonsCommand {
    football_domain::PlanP4HorizonsCommand {
        match_id: Uuid::from_u128(31),
        explicit_rule_package_id: Uuid::from_u128(10),
        requested_fact_keys: vec![],
    }
}

#[tokio::test]
async fn planner_pins_three_formal_horizons_route_schemas_facts_and_queue_policy() {
    let kickoff = Utc::now() + Duration::hours(48);
    let probe = planning_probe(kickoff);
    let tasks = crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
        .await
        .unwrap();
    let state = probe.state.lock().unwrap();
    assert_eq!(
        state.calls,
        [
            "context",
            "scope",
            "route",
            "schema",
            "schema",
            "find_task",
            "create_task",
            "enqueue",
            "transition",
            "find_task",
            "create_task",
            "enqueue",
            "transition",
            "find_task",
            "create_task",
            "enqueue",
            "transition"
        ]
    );
    assert_eq!(tasks.len(), 3);
    assert_eq!(state.schema_reads.len(), 2);
    let request = &state.route_requests[0];
    assert_eq!(request.preferred_model_family.as_deref(), Some("p4"));
    assert_eq!(request.explicit_rule_package_id, Some(Uuid::from_u128(10)));
    assert_eq!(request.kickoff_time, kickoff);
    assert_eq!(
        (request.competition_id, request.season_id, request.stage_id),
        (
            Some(Uuid::from_u128(21)),
            Some(Uuid::from_u128(22)),
            Some(Uuid::from_u128(23))
        )
    );
    for (index, horizon) in P4Horizon::CANONICAL.into_iter().enumerate() {
        let task = &tasks[index];
        let job = &state.enqueues[index];
        let draft = &state.task_drafts[index];
        let cutoff = horizon.data_cutoff_at(kickoff).unwrap();
        assert_eq!(task.horizon, horizon);
        assert_eq!(task.state, P4FreezeTaskState::ResearchQueued);
        assert_eq!(task.requested_fact_keys, canonical_fact_keys());
        assert_eq!(
            (
                task.research_schema_version_id,
                task.snapshot_schema_version_id
            ),
            (Uuid::from_u128(14), Uuid::from_u128(15))
        );
        assert_eq!(
            (
                task.rule_package_id,
                task.model_version_id,
                task.parameter_set_id,
                task.competition_profile_id
            ),
            (
                Uuid::from_u128(10),
                Uuid::from_u128(11),
                Uuid::from_u128(12),
                Uuid::from_u128(13)
            )
        );
        assert_eq!(
            task.idempotency_key,
            format!(
                "p4-freeze:{}:{}:{}:{}:{}:{}",
                task.match_id,
                task.model_version_id,
                task.parameter_set_id,
                task.competition_profile_id,
                horizon.as_str(),
                cutoff.timestamp()
            )
        );
        assert_eq!(task.data_cutoff_at, cutoff);
        assert_eq!(task.research_due_at, cutoff - Duration::minutes(15));
        assert_eq!(task.freeze_deadline_at, cutoff + Duration::minutes(15));
        assert_eq!(draft.state, P4FreezeTaskState::Planned);
        assert_eq!(job.job_type, "p4_horizon_research");
        assert_eq!(job.payload, json!({"task_id":task.id}));
        assert_eq!(
            job.idempotency_key,
            Some(format!("p4-research-job:{}", task.id))
        );
        assert_eq!(job.available_at, Some(task.research_due_at));
        assert_eq!(job.priority, [10, 20, 40][index]);
        assert_eq!(job.max_attempts, 3);
        assert_eq!(
            task.research_job_id,
            Some(Uuid::from_u128(100 + index as u128))
        );
        assert_eq!(
            draft.metadata["planner_version"],
            json!(football_domain::P4_ORCHESTRATION_PLANNER_VERSION)
        );
    }
}

#[tokio::test]
async fn planner_retry_preserves_progressed_and_terminal_tasks_without_writes() {
    let probe = planning_probe(Utc::now() + Duration::hours(48));
    let initial =
        crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
            .await
            .unwrap();
    for preserved in P4FreezeTaskState::ALL
        .into_iter()
        .filter(|s| *s != P4FreezeTaskState::Planned)
    {
        {
            let mut state = probe.state.lock().unwrap();
            state.calls.clear();
            state.planned_tasks[0].state = preserved;
        }
        let retry =
            crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
                .await
                .unwrap();
        let state = probe.state.lock().unwrap();
        assert_eq!(retry[0].state, preserved);
        assert_eq!(retry[0].id, initial[0].id);
        assert_eq!(retry[0].trace_id, initial[0].trace_id);
        assert_eq!(
            state.calls,
            [
                "context",
                "scope",
                "route",
                "schema",
                "schema",
                "find_task",
                "find_task",
                "find_task"
            ]
        );
        assert_eq!(
            (
                state.task_drafts.len(),
                state.enqueues.len(),
                state.transitions.len()
            ),
            (3, 3, 3)
        );
    }
}

#[tokio::test]
async fn planner_rejects_route_capability_or_fact_subset_before_writes() {
    for invalid in ["model", "horizon", "facts"] {
        let probe = planning_probe(Utc::now() + Duration::hours(48));
        let mut command = planning_command();
        {
            let mut state = probe.state.lock().unwrap();
            let route = state.planning_route.as_mut().unwrap();
            match invalid {
                "model" => route.model_id = "p7".into(),
                "horizon" => {
                    route.routing.supported_snapshot_types.pop();
                }
                _ => command.requested_fact_keys = vec!["subset".into()],
            }
        }
        assert!(matches!(
            crate::use_cases::prediction::plan_p4_horizons::execute(&probe, command).await,
            Err(ApplicationError::Validation(_))
        ));
        let state = probe.state.lock().unwrap();
        assert_eq!(state.calls, ["context", "scope", "route"]);
        assert!(state.task_drafts.is_empty());
        assert!(state.enqueues.is_empty());
    }
    let probe = planning_probe(Utc::now() + Duration::hours(48));
    let mut command = planning_command();
    command.requested_fact_keys = canonical_fact_keys()
        .into_iter()
        .map(|s| format!(" {s} "))
        .collect();
    command
        .requested_fact_keys
        .push(command.requested_fact_keys[0].clone());
    let result = crate::use_cases::prediction::plan_p4_horizons::execute(&probe, command)
        .await
        .unwrap();
    assert_eq!(result[0].requested_fact_keys, canonical_fact_keys());
}

#[tokio::test]
async fn planner_rejects_every_pinned_identity_drift_before_resume() {
    for field in [
        "rule_package_id",
        "model_version_id",
        "parameter_set_id",
        "competition_profile_id",
        "research_schema_version_id",
        "snapshot_schema_version_id",
        "requested_fact_keys",
    ] {
        let probe = planning_probe(Utc::now() + Duration::hours(48));
        crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
            .await
            .unwrap();
        {
            let mut state = probe.state.lock().unwrap();
            let mut task = serde_json::to_value(&state.planned_tasks[0]).unwrap();
            task[field] = if field == "requested_fact_keys" {
                json!(["wrong"])
            } else {
                json!(Uuid::from_u128(999))
            };
            state.planned_tasks[0] = serde_json::from_value(task).unwrap();
            state.calls.clear();
        }
        assert!(matches!(
            crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
                .await,
            Err(ApplicationError::Validation(_))
        ));
        let state = probe.state.lock().unwrap();
        assert_eq!(
            state.calls,
            ["context", "scope", "route", "schema", "schema", "find_task"]
        );
        assert_eq!((state.task_drafts.len(), state.enqueues.len()), (3, 3));
    }
}

#[tokio::test]
async fn planner_marks_elapsed_horizons_missed_without_enqueuing_them() {
    for (hours, queued) in [(-1, 0), (2, 1)] {
        let probe = planning_probe(Utc::now() + Duration::hours(hours));
        let result =
            crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
                .await
                .unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(
            result
                .iter()
                .filter(|t| t.state == P4FreezeTaskState::Missed)
                .count(),
            3 - queued
        );
        let state = probe.state.lock().unwrap();
        assert_eq!(state.enqueues.len(), queued);
        assert_eq!(state.transitions.len(), queued);
    }
}

#[tokio::test]
async fn planner_preserves_port_error_kind_and_stops_at_each_boundary() {
    use crate::ports::PortErrorKind;
    for boundary in [
        "context",
        "scope",
        "route",
        "schema",
        "find_task",
        "create_task",
        "enqueue",
        "transition",
    ] {
        for kind in [
            PortErrorKind::Unavailable,
            PortErrorKind::NotFound,
            PortErrorKind::Conflict,
            PortErrorKind::InvalidState,
            PortErrorKind::Serialization,
            PortErrorKind::Infrastructure,
        ] {
            let probe = planning_probe(Utc::now() + Duration::hours(48));
            {
                let mut state = probe.state.lock().unwrap();
                state.failure = Some(boundary);
                state.failure_kind = Some(kind);
            }
            let error =
                crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
                    .await
                    .unwrap_err();
            let ApplicationError::Port(error) = error else {
                panic!("lost Port error")
            };
            assert_eq!(error.kind, kind);
            assert_eq!(error.message, format!("injected {boundary}"));
            assert_eq!(probe.calls().last(), Some(&boundary));
            let state = probe.state.lock().unwrap();
            assert_eq!(
                state.planned_tasks.len(),
                usize::from(matches!(boundary, "enqueue" | "transition"))
            );
            assert_eq!(state.enqueues.len(), usize::from(boundary == "transition"));
            assert!(state.transitions.is_empty());
        }
    }
}

#[tokio::test]
async fn planner_recovers_creation_followed_by_enqueue_failure_with_same_task_identity() {
    let probe = planning_probe(Utc::now() + Duration::hours(48));
    probe.fail_at(Some("enqueue"));
    assert!(
        crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
            .await
            .is_err()
    );
    let original = probe.state.lock().unwrap().planned_tasks[0].clone();
    assert_eq!(original.state, P4FreezeTaskState::Planned);
    probe.fail_at(None);
    let retry = crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
        .await
        .unwrap();
    assert_eq!(
        (
            retry[0].id,
            retry[0].trace_id,
            retry[0].idempotency_key.as_str()
        ),
        (
            original.id,
            original.trace_id,
            original.idempotency_key.as_str()
        )
    );
    assert_eq!(retry[0].state, P4FreezeTaskState::ResearchQueued);
    let state = probe.state.lock().unwrap();
    assert_eq!(
        (
            state.task_drafts.len(),
            state.planned_job_ids.len(),
            state.transitions.len()
        ),
        (3, 3, 3)
    );
}

#[tokio::test]
async fn planner_recovers_binding_failure_using_original_idempotent_queue_job() {
    let probe = planning_probe(Utc::now() + Duration::hours(48));
    probe.fail_at(Some("transition"));
    assert!(
        crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
            .await
            .is_err()
    );
    let (task, job) = {
        let state = probe.state.lock().unwrap();
        (
            state.planned_tasks[0].clone(),
            *state.planned_job_ids.values().next().unwrap(),
        )
    };
    probe.fail_at(None);
    let retry = crate::use_cases::prediction::plan_p4_horizons::execute(&probe, planning_command())
        .await
        .unwrap();
    assert_eq!(retry[0].id, task.id);
    assert_eq!(retry[0].research_job_id, Some(job));
    let state = probe.state.lock().unwrap();
    assert_eq!(
        state.enqueues[0].idempotency_key,
        state.enqueues[1].idempotency_key
    );
    assert_eq!(
        (
            state.task_drafts.len(),
            state.enqueues.len(),
            state.planned_job_ids.len()
        ),
        (3, 4, 3)
    );
}
