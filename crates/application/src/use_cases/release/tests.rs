use super::{
    list_release_acceptance_runs, read_release_acceptance_run,
    run_acceptance::run_release_acceptance,
};
use crate::{
    ports::{release::ReleaseAcceptancePort, PortError, PortErrorKind, PortResult},
    ApplicationError,
};
use async_trait::async_trait;
use football_domain::{
    ReleaseAcceptanceRequest, ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary,
    ReleaseAcceptanceRuntimeFacts, ReleaseAcceptanceStatus,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use uuid::Uuid;

fn facts() -> ReleaseAcceptanceRuntimeFacts {
    ReleaseAcceptanceRuntimeFacts {
        migration_count: 46,
        database_latency_ms: 100,
        integration_stages: ('A'..='J').map(|stage| stage.to_string()).collect(),
        immutable_trigger_count: 8,
        provider_boundary_artifact_count: 1,
        freeze_task_count: 1,
        frozen_snapshot_count: 1,
        settlement_count: 1,
        evidence_decision_count: 1,
        shadow_validation_count: 1,
        promotion_decision_count: 1,
        recent_model_run_count: 20,
        recent_model_run_p95_ms: Some(100.0),
        recent_model_failure_count: 0,
        query_warning_count: 0,
        completed_requests: 4,
        failed_requests: 0,
        search_calls: 3,
        estimated_cost_usd: 1.0,
        latest_day_cost_usd: 0.5,
    }
}
fn request() -> ReleaseAcceptanceRequest {
    ReleaseAcceptanceRequest {
        daily_cost_budget_usd: Some(1.0),
        monthly_cost_budget_usd: Some(2.0),
        requested_by: Some(" reviewer ".into()),
        ..Default::default()
    }
}
fn stored_run() -> ReleaseAcceptanceRun {
    serde_json::from_value(json!({"id": Uuid::from_u128(1), "app_version": "stored app",
        "contract_version": "stored contract", "fixture_version": "stored fixture", "overall_status": "blocked",
        "started_at": "2026-10-14T00:00:00Z", "completed_at": "2026-10-14T00:00:01Z",
        "requested_by": null, "report_sha256": "stored opaque hash", "passed_count": 2,
        "warning_count": 3, "blocked_count": 4, "category_summaries": [],
        "performance": {"database_latency_ms": 200, "recent_model_run_count": 1, "recent_model_run_p95_ms": null,
            "recent_model_failure_count": 0, "query_warning_count": 0},
        "cost": {"window_days": 17, "completed_requests": 1, "failed_requests": 2, "search_calls": 3,
            "estimated_cost_usd": 1.0, "latest_day_cost_usd": 0.5, "daily_budget_usd": null, "monthly_budget_usd": null},
        "checks": []})).unwrap()
}
fn stored_summary() -> ReleaseAcceptanceRunSummary {
    serde_json::from_value(json!({"id": Uuid::from_u128(1), "app_version": "stored app", "overall_status": "blocked",
        "completed_at": "2026-10-14T00:00:01Z", "requested_by": null, "passed_count": 2, "warning_count": 3,
        "blocked_count": 4, "report_sha256": "stored opaque hash"})).unwrap()
}
struct State {
    facts: ReleaseAcceptanceRuntimeFacts,
    calls: Mutex<Vec<Value>>,
    fail_at: Option<&'static str>,
    error_kind: PortErrorKind,
}
#[derive(Clone)]
struct Fixture {
    state: Arc<State>,
}
impl Fixture {
    fn new(facts: ReleaseAcceptanceRuntimeFacts) -> Self {
        Self::failing(facts, None, PortErrorKind::Infrastructure)
    }
    fn failing(
        facts: ReleaseAcceptanceRuntimeFacts,
        fail_at: Option<&'static str>,
        error_kind: PortErrorKind,
    ) -> Self {
        Self {
            state: Arc::new(State {
                facts,
                calls: Mutex::new(Vec::new()),
                fail_at,
                error_kind,
            }),
        }
    }
    fn note(&self, method: &'static str, data: Value) -> PortResult<()> {
        self.state
            .calls
            .lock()
            .unwrap()
            .push(json!({"method": method, "data": data}));
        if self.state.fail_at == Some(method) {
            return Err(PortError::new(
                self.state.error_kind,
                format!("injected {method}"),
            ));
        }
        Ok(())
    }
    fn calls(&self) -> Vec<Value> {
        self.state.calls.lock().unwrap().clone()
    }
    fn names(&self) -> Vec<String> {
        self.calls()
            .iter()
            .map(|call| call["method"].as_str().unwrap().to_string())
            .collect()
    }
}
#[async_trait]
impl ReleaseAcceptancePort for Fixture {
    async fn runtime_facts(
        &self,
        performance_window_days: u32,
        cost_window_days: u32,
    ) -> PortResult<ReleaseAcceptanceRuntimeFacts> {
        self.note(
            "facts",
            json!({"performance": performance_window_days, "cost": cost_window_days}),
        )?;
        Ok(self.state.facts.clone())
    }
    async fn persist_run(&self, run: &ReleaseAcceptanceRun) -> PortResult<()> {
        self.note("persist", json!(run))
    }
    async fn list_runs(&self, limit: u32) -> PortResult<Vec<ReleaseAcceptanceRunSummary>> {
        self.note("list", json!({"limit": limit}))?;
        Ok(vec![stored_summary()])
    }
    async fn read_run(&self, run_id: Uuid) -> PortResult<ReleaseAcceptanceRun> {
        self.note("read", json!({"id": run_id}))?;
        Ok(stored_run())
    }
}
fn reference_hash(run: &ReleaseAcceptanceRun) -> String {
    let mut value = serde_json::to_value(run).unwrap();
    let object = value.as_object_mut().unwrap();
    let id = object.remove("id").unwrap();
    object.insert("run_id".into(), id);
    object.remove("report_sha256");
    hex::encode(Sha256::digest(serde_json::to_vec(&value).unwrap()))
}
fn status(run: &ReleaseAcceptanceRun, code: &str) -> ReleaseAcceptanceStatus {
    run.checks
        .iter()
        .find(|check| check.check_code == code)
        .unwrap()
        .status
}
fn kinds() -> [PortErrorKind; 6] {
    [
        PortErrorKind::Unavailable,
        PortErrorKind::NotFound,
        PortErrorKind::Conflict,
        PortErrorKind::InvalidState,
        PortErrorKind::Serialization,
        PortErrorKind::Infrastructure,
    ]
}

#[tokio::test]
async fn run_normalizes_windows_and_persists_exact_completed_report_before_return() {
    let fixture = Fixture::new(facts());
    let mut draft = request();
    draft.performance_window_days = 0;
    draft.cost_window_days = u32::MAX;
    let run = run_release_acceptance(async { Ok(fixture.clone()) }, draft)
        .await
        .unwrap();
    assert_eq!(fixture.names(), ["facts", "persist"]);
    assert_eq!(
        fixture.calls()[0]["data"],
        json!({"performance": 1, "cost": 365})
    );
    assert_eq!(fixture.calls()[1]["data"], json!(run));
    assert_eq!(run.cost.window_days, 365);
    assert_eq!(run.requested_by.as_deref(), Some(" reviewer "));
    assert!(run.completed_at >= run.started_at);
    assert_eq!(run.report_sha256, reference_hash(&run));
}

#[tokio::test]
async fn invalid_budgets_fail_before_polling_lazy_session_and_keep_daily_first_error() {
    for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for daily in [true, false] {
            let fixture = Fixture::new(facts());
            let acquired = Arc::new(AtomicBool::new(false));
            let flag = acquired.clone();
            let port = fixture.clone();
            let session = async move {
                flag.store(true, Ordering::SeqCst);
                Ok(port)
            };
            let mut draft = request();
            if daily {
                draft.daily_cost_budget_usd = Some(invalid);
                draft.monthly_cost_budget_usd = Some(invalid);
            } else {
                draft.monthly_cost_budget_usd = Some(invalid);
            }
            let label = if daily {
                "单日成本预算"
            } else {
                "周期成本预算"
            };
            assert!(
                matches!(run_release_acceptance(session,draft).await.unwrap_err(),ApplicationError::Validation(message)
                if message==format!("{label}必须是大于或等于 0 的有限数值"))
            );
            assert!(!acquired.load(Ordering::SeqCst));
            assert!(fixture.calls().is_empty());
        }
    }
}

#[tokio::test]
async fn session_failure_preserves_original_error_for_all_three_entries() {
    let error = || ApplicationError::Validation("session failure".into());
    assert!(
        matches!(run_release_acceptance(async {Err::<Fixture,_>(error())},request()).await.unwrap_err(),ApplicationError::Validation(message) if message=="session failure")
    );
    assert!(
        matches!(list_release_acceptance_runs(async {Err::<Fixture,_>(error())},0).await.unwrap_err(),ApplicationError::Validation(message) if message=="session failure")
    );
    assert!(
        matches!(read_release_acceptance_run(async {Err::<Fixture,_>(error())},Uuid::nil()).await.unwrap_err(),ApplicationError::Validation(message) if message=="session failure")
    );
}

#[tokio::test]
async fn runtime_port_errors_preserve_all_kinds_and_stop_before_report_write() {
    for kind in kinds() {
        let fixture = Fixture::failing(facts(), Some("facts"), kind);
        assert!(
            matches!(run_release_acceptance(async {Ok(fixture.clone())},request()).await.unwrap_err(),
            ApplicationError::Port(error) if error.kind==kind && error.message=="injected facts")
        );
        assert_eq!(fixture.names(), ["facts"]);
    }
}

#[tokio::test]
async fn persist_errors_preserve_all_kinds_without_success_receipt_or_retry() {
    for kind in kinds() {
        let fixture = Fixture::failing(facts(), Some("persist"), kind);
        assert!(
            matches!(run_release_acceptance(async {Ok(fixture.clone())},request()).await.unwrap_err(),
            ApplicationError::Port(error) if error.kind==kind && error.message=="injected persist")
        );
        assert_eq!(fixture.names(), ["facts", "persist"]);
        let attempted: ReleaseAcceptanceRun =
            serde_json::from_value(fixture.calls()[1]["data"].clone()).unwrap();
        assert_eq!(attempted.report_sha256, reference_hash(&attempted));
    }
}

#[tokio::test]
async fn repeated_runs_keep_distinct_report_identity_and_prior_receipt_unchanged() {
    let fixture = Fixture::new(facts());
    let first = run_release_acceptance(async { Ok(fixture.clone()) }, request())
        .await
        .unwrap();
    let snapshot = json!(first);
    let second = run_release_acceptance(async { Ok(fixture.clone()) }, request())
        .await
        .unwrap();
    assert_ne!(first.id, second.id);
    assert_ne!(first.report_sha256, second.report_sha256);
    assert_eq!(fixture.names(), ["facts", "persist", "facts", "persist"]);
    assert_eq!(fixture.calls()[1]["data"], snapshot);
    assert_eq!(fixture.calls()[3]["data"], json!(second));
}

#[tokio::test]
async fn original_check_order_identity_categories_and_public_provider_warning_remain() {
    let fixture = Fixture::new(facts());
    let run = run_release_acceptance(async { Ok(fixture.clone()) }, request())
        .await
        .unwrap();
    let codes: Vec<_> = run
        .checks
        .iter()
        .map(|check| check.check_code.as_str())
        .collect();
    assert_eq!(
        codes,
        [
            "integration_contracts_a_to_i",
            "external_model_provider_boundary",
            "database_migrations",
            "runtime_lifecycle_evidence",
            "public_model_boundary",
            "external_model_runtime",
            "database_latency",
            "model_run_performance",
            "database_query_health",
            "immutable_ledgers",
            "credential_boundary",
            "openai_cost_observability",
            "release_artifact_contract"
        ]
    );
    for (index, check) in run.checks.iter().enumerate() {
        assert_eq!(check.run_id, run.id);
        assert_eq!(check.sequence_no, index as i32 + 1);
    }
    let categories: Vec<_> = run
        .category_summaries
        .iter()
        .map(|summary| summary.category.as_str())
        .collect();
    assert_eq!(
        categories,
        ["chain", "cost", "performance", "release", "security"]
    );
    assert_eq!(
        (run.passed_count, run.warning_count, run.blocked_count),
        (12, 1, 0)
    );
    assert_eq!(run.overall_status, ReleaseAcceptanceStatus::Warning);
    assert_eq!(
        status(&run, "external_model_runtime"),
        ReleaseAcceptanceStatus::Warning
    );
}

#[tokio::test]
async fn cost_gate_keeps_missing_budget_warning_and_strict_exceeded_comparison() {
    for (daily, period, expected) in [
        (Some(0.5), Some(1.0), ReleaseAcceptanceStatus::Pass),
        (Some(0.0), Some(1.0), ReleaseAcceptanceStatus::Blocked),
        (Some(0.5), Some(0.0), ReleaseAcceptanceStatus::Blocked),
        (None, Some(1.0), ReleaseAcceptanceStatus::Warning),
        (Some(0.5), None, ReleaseAcceptanceStatus::Warning),
    ] {
        let fixture = Fixture::new(facts());
        let mut draft = request();
        draft.daily_cost_budget_usd = daily;
        draft.monthly_cost_budget_usd = period;
        let run = run_release_acceptance(async { Ok(fixture.clone()) }, draft)
            .await
            .unwrap();
        assert_eq!(status(&run, "openai_cost_observability"), expected);
        assert_eq!(run.cost.daily_budget_usd, daily);
        assert_eq!(run.cost.monthly_budget_usd, period);
        assert_eq!(fixture.names(), ["facts", "persist"]);
    }
}

#[tokio::test]
async fn performance_gate_keeps_original_strict_latency_and_p95_boundaries() {
    for (latency, expected) in [
        (500, ReleaseAcceptanceStatus::Pass),
        (501, ReleaseAcceptanceStatus::Warning),
        (1500, ReleaseAcceptanceStatus::Warning),
        (1501, ReleaseAcceptanceStatus::Blocked),
    ] {
        let mut input = facts();
        input.database_latency_ms = latency;
        let fixture = Fixture::new(input);
        let run = run_release_acceptance(async { Ok(fixture.clone()) }, request())
            .await
            .unwrap();
        assert_eq!(status(&run, "database_latency"), expected);
    }
    for (p95, expected) in [
        (Some(2000.0), ReleaseAcceptanceStatus::Pass),
        (Some(2001.0), ReleaseAcceptanceStatus::Warning),
        (Some(5000.0), ReleaseAcceptanceStatus::Warning),
        (Some(5001.0), ReleaseAcceptanceStatus::Blocked),
        (None, ReleaseAcceptanceStatus::Warning),
    ] {
        let mut input = facts();
        input.recent_model_run_p95_ms = p95;
        let fixture = Fixture::new(input);
        let run = run_release_acceptance(async { Ok(fixture.clone()) }, request())
            .await
            .unwrap();
        assert_eq!(status(&run, "model_run_performance"), expected);
    }
}

#[tokio::test]
async fn blocked_chain_security_and_j_checks_are_recorded_without_claiming_release_pass() {
    let mut input = facts();
    input.integration_stages = vec!["A".into()];
    input.migration_count = 26;
    input.immutable_trigger_count = 7;
    input.frozen_snapshot_count = 0;
    input.settlement_count = 0;
    let fixture = Fixture::new(input);
    let run = run_release_acceptance(async { Ok(fixture.clone()) }, request())
        .await
        .unwrap();
    for code in [
        "integration_contracts_a_to_i",
        "database_migrations",
        "immutable_ledgers",
        "release_artifact_contract",
    ] {
        assert_eq!(status(&run, code), ReleaseAcceptanceStatus::Blocked);
    }
    assert_eq!(
        status(&run, "runtime_lifecycle_evidence"),
        ReleaseAcceptanceStatus::Warning
    );
    assert_eq!(run.overall_status, ReleaseAcceptanceStatus::Blocked);
    assert_eq!(fixture.names(), ["facts", "persist"]);
    assert_eq!(fixture.calls()[1]["data"], json!(run));
}

#[tokio::test]
async fn summary_keeps_original_zero_projection_for_negative_runtime_counters() {
    let mut input = facts();
    input.recent_model_run_count = -1;
    input.recent_model_failure_count = -2;
    input.query_warning_count = -3;
    input.completed_requests = -4;
    input.failed_requests = -5;
    input.search_calls = -6;
    let fixture = Fixture::new(input);
    let run = run_release_acceptance(async { Ok(fixture.clone()) }, request())
        .await
        .unwrap();
    assert_eq!(run.performance.recent_model_run_count, 0);
    assert_eq!(run.performance.recent_model_failure_count, 0);
    assert_eq!(run.performance.query_warning_count, 0);
    assert_eq!(
        (
            run.cost.completed_requests,
            run.cost.failed_requests,
            run.cost.search_calls
        ),
        (0, 0, 0)
    );
    assert_eq!(run.report_sha256, reference_hash(&run));
}

#[tokio::test]
async fn report_hash_binds_public_receipt_fields_and_check_evidence() {
    let fixture = Fixture::new(facts());
    let run = run_release_acceptance(async { Ok(fixture.clone()) }, request())
        .await
        .unwrap();
    assert_eq!(run.report_sha256, reference_hash(&run));
    for index in 0..4 {
        let mut altered = run.clone();
        match index {
            0 => altered.requested_by = Some("another reviewer".into()),
            1 => altered.passed_count += 1,
            2 => altered.cost.window_days += 1,
            _ => altered.checks[0].evidence = json!({"altered":true}),
        }
        assert_ne!(run.report_sha256, reference_hash(&altered));
    }
    assert_eq!(fixture.calls()[1]["data"], json!(run));
}

#[tokio::test]
async fn list_query_forwards_zero_and_max_limits_and_returns_original_summaries() {
    for limit in [0, u32::MAX] {
        let fixture = Fixture::new(facts());
        let result = list_release_acceptance_runs(async { Ok(fixture.clone()) }, limit)
            .await
            .unwrap();
        assert_eq!(json!(result), json!([stored_summary()]));
        assert_eq!(fixture.names(), ["list"]);
        assert_eq!(fixture.calls()[0]["data"], json!({"limit":limit}));
    }
}

#[tokio::test]
async fn read_query_preserves_identity_and_opaque_stored_report_without_rehashing() {
    let fixture = Fixture::new(facts());
    let result = read_release_acceptance_run(async { Ok(fixture.clone()) }, Uuid::nil())
        .await
        .unwrap();
    assert_eq!(json!(result), json!(stored_run()));
    assert_eq!(fixture.names(), ["read"]);
    assert_eq!(fixture.calls()[0]["data"], json!({"id":Uuid::nil()}));
}

#[tokio::test]
async fn report_query_errors_preserve_all_kinds_without_other_reads_or_retry() {
    for kind in kinds() {
        let fixture = Fixture::failing(facts(), Some("list"), kind);
        assert!(
            matches!(list_release_acceptance_runs(async {Ok(fixture.clone())},0).await.unwrap_err(),ApplicationError::Port(error) if error.kind==kind)
        );
        assert_eq!(fixture.names(), ["list"]);
        let fixture = Fixture::failing(facts(), Some("read"), kind);
        assert!(
            matches!(read_release_acceptance_run(async {Ok(fixture.clone())},Uuid::nil()).await.unwrap_err(),ApplicationError::Port(error) if error.kind==kind)
        );
        assert_eq!(fixture.names(), ["read"]);
    }
}
