use super::{
    cancel_background_job, coordinator::run_queue, enqueue_analysis_job, execution::execute_job,
    list_background_jobs, request::parse_refresh_request, retry_background_job,
};
use crate::{
    ports::{
        analytics::{AnalyticsJobProgressPayload, AnalyticsJobResult, AnalyticsPort, JobQueuePort},
        PortError, PortErrorKind, PortResult,
    },
    ApplicationError,
};
use async_trait::async_trait;
use football_domain::{
    AiAnalysisPackageData, AiAnalysisPackageSummary, AiAnalysisResponsePreview,
    AiAnalysisSuggestionRecord, AiSuggestionDecisionDraft, AnalyticsOverview,
    AnalyticsRefreshRequest, BackgroundJob, DataQualityDecisionDraft, DataQualityFinding,
    DataQualitySummary, EnqueueJobDraft, JobStatus, QueryPerformanceSummary,
};
use serde_json::{json, Value};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Notify;
use uuid::Uuid;

fn response() -> Value {
    json!({
        "generated_at": "2026-10-13T00:00:00.123456789Z", "calculation_version": "original-version",
        "sample_size": 17, "average_log_loss": 0.3, "average_brier": 0.2,
        "average_scoreline_nll": null, "expected_calibration_error": 0.125,
        "comparisons": [{"model_key": "fixture", "model_version": "v1", "parameter_version": "p1",
            "snapshot_type": "T-1h", "sample_size": 17, "average_log_loss": 0.3, "average_brier": 0.2,
            "average_scoreline_nll": null, "average_data_coverage": 0.75, "rank": 2}],
        "calibration": [{"outcome": "draw", "bucket_index": 9, "lower_bound": 0.9, "upper_bound": 1.0,
            "sample_size": 3, "predicted_mean": 0.95, "actual_rate": 1.0, "absolute_gap": 0.05, "ece_component": 0.01}],
        "drift": [{"metric_name": "log_loss", "baseline_mean": 0.5, "current_mean": 0.7,
            "absolute_delta": 0.2, "relative_delta": null, "baseline_size": 5, "current_size": 5,
            "severity": "warning", "direction": "up"}],
        "data_quality": {"scan_id": Uuid::from_u128(7), "generated_at": null,
            "critical": 2, "warning": 1, "info": 0, "open_total": 3, "findings": []},
        "query_performance": {"captured_at": null, "database_size_bytes": 12345, "tables": []}
    })
}

fn job(index: u128, kind: &str) -> BackgroundJob {
    BackgroundJob {
        id: Uuid::from_u128(index),
        job_type: kind.into(),
        status: JobStatus::Queued,
        progress: 0.0,
        payload: json!({}),
        result: None,
        error_message: None,
        priority: 7,
        attempts: 2,
        max_attempts: 3,
        cancellation_requested: false,
        available_at: "2026-10-13T00:00:00.123456789Z".parse().unwrap(),
        created_at: "2026-10-13T00:00:00.123456789Z".parse().unwrap(),
        started_at: None,
        finished_at: None,
        updated_at: "2026-10-13T00:00:00.123456789Z".parse().unwrap(),
    }
}

struct State {
    calls: Vec<Value>,
    claims: VecDeque<PortResult<Option<BackgroundJob>>>,
    fail_at: Vec<&'static str>,
    error_kind: PortErrorKind,
    cancel_at: Option<f64>,
    output: BackgroundJob,
}
#[derive(Clone)]
struct Fixture {
    state: Arc<Mutex<State>>,
    idle: Arc<Notify>,
}
impl Fixture {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                calls: Vec::new(),
                claims: VecDeque::new(),
                fail_at: Vec::new(),
                error_kind: PortErrorKind::Infrastructure,
                cancel_at: None,
                output: job(77, "refresh_analytics"),
            })),
            idle: Arc::new(Notify::new()),
        }
    }
    fn note(&self, call: &'static str, data: Value) -> PortResult<()> {
        let mut state = self.state.lock().unwrap();
        state.calls.push(json!({"call": call, "data": data}));
        if state.fail_at.contains(&call) {
            return Err(PortError::new(state.error_kind, format!("injected {call}")));
        }
        Ok(())
    }
    fn calls(&self) -> Vec<Value> {
        self.state.lock().unwrap().calls.clone()
    }
    fn names(&self) -> Vec<String> {
        self.calls()
            .iter()
            .map(|call| call["call"].as_str().unwrap().to_string())
            .collect()
    }
    fn output(&self) -> BackgroundJob {
        self.state.lock().unwrap().output.clone()
    }
    async fn wait_idle(&self) {
        tokio::time::timeout(Duration::from_secs(5), self.idle.notified())
            .await
            .expect("worker reaches empty queue");
    }
}
#[async_trait]
impl AnalyticsPort for Fixture {
    async fn overview(&self) -> PortResult<AnalyticsOverview> {
        panic!("worker must use refresh result")
    }
    async fn refresh(&self, request: &AnalyticsRefreshRequest) -> PortResult<AnalyticsOverview> {
        self.note("refresh", serde_json::to_value(request).unwrap())?;
        Ok(serde_json::from_value(response()).unwrap())
    }
    async fn run_data_quality_scan(&self) -> PortResult<DataQualitySummary> {
        self.note("quality", json!({}))?;
        Ok(serde_json::from_value(response()["data_quality"].clone()).unwrap())
    }
    async fn capture_query_performance(&self) -> PortResult<QueryPerformanceSummary> {
        self.note("query", json!({}))?;
        Ok(serde_json::from_value(response()["query_performance"].clone()).unwrap())
    }
    async fn decide_data_quality(
        &self,
        _draft: &DataQualityDecisionDraft,
    ) -> PortResult<DataQualityFinding> {
        panic!("unexpected manual quality decision")
    }
    async fn build_ai_analysis_data(&self) -> PortResult<AiAnalysisPackageData> {
        panic!("unexpected AI read")
    }
    async fn record_ai_export(&self, _summary: &AiAnalysisPackageSummary) -> PortResult<()> {
        panic!("unexpected AI export")
    }
    async fn import_ai_response(
        &self,
        _input_path: &str,
        _preview: &AiAnalysisResponsePreview,
    ) -> PortResult<Vec<AiAnalysisSuggestionRecord>> {
        panic!("unexpected AI import")
    }
    async fn list_ai_suggestions(
        &self,
        _status: Option<&str>,
        _limit: u32,
    ) -> PortResult<Vec<AiAnalysisSuggestionRecord>> {
        panic!("unexpected AI suggestions")
    }
    async fn decide_ai_suggestion(
        &self,
        _draft: &AiSuggestionDecisionDraft,
    ) -> PortResult<AiAnalysisSuggestionRecord> {
        panic!("unexpected AI decision")
    }
}
#[async_trait]
impl JobQueuePort for Fixture {
    async fn enqueue(&self, draft: &EnqueueJobDraft) -> PortResult<BackgroundJob> {
        self.note("enqueue", serde_json::to_value(draft).unwrap())?;
        Ok(self.output())
    }
    async fn list_jobs(&self, limit: u32) -> PortResult<Vec<BackgroundJob>> {
        self.note("list", json!(limit))?;
        Ok(vec![self.output()])
    }
    async fn request_cancellation(&self, id: Uuid) -> PortResult<BackgroundJob> {
        self.note("cancel", json!(id))?;
        Ok(self.output())
    }
    async fn retry(&self, id: Uuid) -> PortResult<BackgroundJob> {
        self.note("retry", json!(id))?;
        Ok(self.output())
    }
    async fn claim_next_by_types(&self, types: &[&str]) -> PortResult<Option<BackgroundJob>> {
        self.note("claim", json!(types))?;
        let next = self
            .state
            .lock()
            .unwrap()
            .claims
            .pop_front()
            .unwrap_or(Ok(None));
        if !matches!(&next, Ok(Some(_))) {
            self.idle.notify_one();
        }
        next
    }
    async fn update_progress(
        &self,
        id: Uuid,
        progress: f64,
        message: &str,
        payload: AnalyticsJobProgressPayload,
    ) -> PortResult<bool> {
        self.note("progress", json!({"id": id, "progress": progress, "message": message, "payload": payload.into_value()}))?;
        Ok(self.state.lock().unwrap().cancel_at == Some(progress))
    }
    async fn complete(&self, id: Uuid, result: AnalyticsJobResult) -> PortResult<()> {
        self.note(
            "complete",
            json!({"id": id, "result": result.into_value().unwrap()}),
        )
    }
    async fn fail(&self, id: Uuid, error: &str) -> PortResult<()> {
        self.note("fail", json!({"id": id, "error": error}))
    }
}

#[test]
fn request_defaults_partial_fields_and_nanoseconds_remain_original() {
    for input in [Value::Null, json!({})] {
        assert_eq!(
            serde_json::to_value(parse_refresh_request(&input).unwrap()).unwrap(),
            serde_json::to_value(AnalyticsRefreshRequest::default()).unwrap()
        );
    }
    let input = json!({"competition_id": Uuid::from_u128(9), "window_start": "2026-10-13T00:00:00.123456789Z", "bucket_count": 0, "unknown": "ignored"});
    let request = parse_refresh_request(&input).unwrap();
    assert_eq!(request.competition_id, Some(Uuid::from_u128(9)));
    assert_eq!(
        request.window_start.unwrap().to_rfc3339(),
        "2026-10-13T00:00:00.123456789+00:00"
    );
    assert!(request.window_end.is_none());
    assert_eq!(
        (
            request.bucket_count,
            request.baseline_size,
            request.current_size
        ),
        (0, 100, 50)
    );
}

#[tokio::test]
async fn malformed_refresh_payload_stops_before_progress_or_work() {
    for kind in ["refresh_analytics", "full_analysis_refresh"] {
        for payload in [
            json!(false),
            json!(1),
            json!(["bad"]),
            json!({"bucket_count": 256}),
            json!({"window_start": "bad"}),
        ] {
            let fixture = Fixture::new();
            assert!(matches!(
                execute_job(&fixture, kind, &payload, Uuid::nil())
                    .await
                    .unwrap_err(),
                ApplicationError::Json(_)
            ));
            assert!(fixture.calls().is_empty());
        }
    }
}

#[tokio::test]
async fn full_refresh_preserves_progress_payloads_order_request_and_result() {
    let fixture = Fixture::new();
    let id = Uuid::from_u128(99);
    let payload = json!({"bucket_count": 4, "baseline_size": 7, "current_size": 8});
    let result = execute_job(&fixture, "full_analysis_refresh", &payload, id)
        .await
        .unwrap()
        .into_value()
        .unwrap();
    assert_eq!(
        fixture.names(),
        ["progress", "refresh", "progress", "quality", "progress", "query"]
    );
    let calls = fixture.calls();
    for (index, progress, message, data) in [
        (0, 10.0, "正在刷新模型评估", json!({})),
        (2, 55.0, "正在扫描数据质量", json!({"sample_size": 17})),
        (4, 82.0, "正在分析数据库查询", json!({"open_findings": 3})),
    ] {
        assert_eq!(
            calls[index]["data"],
            json!({"id": id, "progress": progress, "message": message, "payload": data})
        );
    }
    assert_eq!(
        calls[1]["data"],
        serde_json::to_value(parse_refresh_request(&payload).unwrap()).unwrap()
    );
    assert_eq!(
        result,
        json!({"sample_size": 17, "expected_calibration_error": 0.125, "quality_findings": 3, "database_size_bytes": 12345})
    );
}

#[tokio::test]
async fn standalone_jobs_preserve_initial_checkpoint_and_complete_typed_result() {
    for (kind, progress, operation, expected) in [
        ("refresh_analytics", 15.0, "refresh", response()),
        (
            "data_quality_scan",
            20.0,
            "quality",
            response()["data_quality"].clone(),
        ),
        (
            "query_performance_scan",
            20.0,
            "query",
            response()["query_performance"].clone(),
        ),
    ] {
        let fixture = Fixture::new();
        let result = execute_job(&fixture, kind, &Value::Null, Uuid::nil())
            .await
            .unwrap()
            .into_value()
            .unwrap();
        assert_eq!(result, expected);
        assert_eq!(fixture.names(), ["progress", operation]);
        assert_eq!(fixture.calls()[0]["data"]["progress"], json!(progress));
    }
}

#[tokio::test]
async fn cancellation_at_each_checkpoint_stops_all_later_side_effects() {
    for (kind, progress, expected) in [
        ("refresh_analytics", 15.0, vec!["progress"]),
        ("data_quality_scan", 20.0, vec!["progress"]),
        ("query_performance_scan", 20.0, vec!["progress"]),
        ("full_analysis_refresh", 10.0, vec!["progress"]),
        (
            "full_analysis_refresh",
            55.0,
            vec!["progress", "refresh", "progress"],
        ),
        (
            "full_analysis_refresh",
            82.0,
            vec!["progress", "refresh", "progress", "quality", "progress"],
        ),
    ] {
        let fixture = Fixture::new();
        fixture.state.lock().unwrap().cancel_at = Some(progress);
        assert!(
            matches!(execute_job(&fixture, kind, &Value::Null, Uuid::nil()).await.unwrap_err(), ApplicationError::Validation(message) if message == "任务已取消")
        );
        assert_eq!(fixture.names(), expected);
    }
}

#[tokio::test]
async fn work_and_progress_errors_propagate_without_later_stages() {
    for (operation, expected) in [
        ("progress", vec!["progress"]),
        ("refresh", vec!["progress", "refresh"]),
        (
            "quality",
            vec!["progress", "refresh", "progress", "quality"],
        ),
        (
            "query",
            vec![
                "progress", "refresh", "progress", "quality", "progress", "query",
            ],
        ),
    ] {
        let fixture = Fixture::new();
        fixture.state.lock().unwrap().fail_at.push(operation);
        assert!(
            matches!(execute_job(&fixture, "full_analysis_refresh", &Value::Null, Uuid::nil()).await.unwrap_err(), ApplicationError::Port(error) if error.kind == PortErrorKind::Infrastructure && error.message == format!("injected {operation}"))
        );
        assert_eq!(fixture.names(), expected);
    }
}

#[tokio::test]
async fn unsupported_job_kind_is_rejected_before_any_port_call() {
    let fixture = Fixture::new();
    assert!(
        matches!(execute_job(&fixture, "p4_horizon_research", &Value::Null, Uuid::nil()).await.unwrap_err(), ApplicationError::Validation(message) if message == "不支持的后台任务：p4_horizon_research")
    );
    assert!(fixture.calls().is_empty());
}

#[tokio::test]
async fn coordinator_claims_only_original_four_types_and_completes_each_job() {
    let fixture = Fixture::new();
    let kinds = [
        "refresh_analytics",
        "data_quality_scan",
        "query_performance_scan",
        "full_analysis_refresh",
    ];
    for (i, kind) in kinds.iter().enumerate() {
        fixture
            .state
            .lock()
            .unwrap()
            .claims
            .push_back(Ok(Some(job(i as u128 + 1, kind))));
    }
    run_queue(&fixture).await;
    let calls = fixture.calls();
    let claims: Vec<_> = calls.iter().filter(|c| c["call"] == "claim").collect();
    assert_eq!(claims.len(), 5);
    for claim in claims {
        assert_eq!(claim["data"], json!(kinds));
    }
    let completed: Vec<_> = calls.iter().filter(|c| c["call"] == "complete").collect();
    assert_eq!(completed.len(), 4);
    for (i, call) in completed.iter().enumerate() {
        assert_eq!(call["data"]["id"], json!(Uuid::from_u128(i as u128 + 1)));
    }
    assert_eq!(completed[0]["data"]["result"], response());
    assert!(!fixture.names().iter().any(|name| name == "fail"));
}

#[tokio::test]
async fn coordinator_records_execution_failure_and_ignores_settlement_errors_before_next_claim() {
    let fixture = Fixture::new();
    {
        let mut state = fixture.state.lock().unwrap();
        state.fail_at = vec!["refresh", "complete", "fail"];
        state
            .claims
            .push_back(Ok(Some(job(1, "refresh_analytics"))));
        state
            .claims
            .push_back(Ok(Some(job(2, "data_quality_scan"))));
    }
    run_queue(&fixture).await;
    assert_eq!(
        fixture.names(),
        [
            "claim", "progress", "refresh", "fail", "claim", "progress", "quality", "complete",
            "claim"
        ]
    );
    let calls = fixture.calls();
    let expected = ApplicationError::Port(PortError::new(
        PortErrorKind::Infrastructure,
        "injected refresh",
    ))
    .to_string();
    assert_eq!(
        calls[3]["data"],
        json!({"id": Uuid::from_u128(1), "error": expected})
    );
    assert_eq!(calls[7]["data"]["id"], json!(Uuid::from_u128(2)));
}

#[tokio::test]
async fn empty_or_failed_claim_ends_worker_without_execution_or_settlement() {
    for claim in [
        Ok(None),
        Err(PortError::new(
            PortErrorKind::Unavailable,
            "original claim failure",
        )),
    ] {
        let fixture = Fixture::new();
        fixture.state.lock().unwrap().claims.push_back(claim);
        run_queue(&fixture).await;
        assert_eq!(fixture.names(), ["claim"]);
    }
}

#[tokio::test]
async fn commands_preserve_input_and_start_worker_only_after_enqueue_or_retry_success() {
    let fixture = Fixture::new();
    let draft: EnqueueJobDraft = serde_json::from_value(json!({"job_type": "full_analysis_refresh", "payload": {"opaque": [1,2]},
        "idempotency_key": "original-key", "available_at": "2026-10-13T00:00:00.123456789Z", "priority": i32::MAX, "max_attempts": 0})).unwrap();
    let expected = serde_json::to_value(&draft).unwrap();
    let result = enqueue_analysis_job(&fixture, draft).await.unwrap();
    fixture.wait_idle().await;
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        serde_json::to_value(fixture.output()).unwrap()
    );
    assert_eq!(fixture.names(), ["enqueue", "claim"]);
    assert_eq!(fixture.calls()[0]["data"], expected);
    let fixture = Fixture::new();
    let id = Uuid::from_u128(88);
    let result = retry_background_job(&fixture, id).await.unwrap();
    fixture.wait_idle().await;
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        serde_json::to_value(fixture.output()).unwrap()
    );
    assert_eq!(fixture.names(), ["retry", "claim"]);
    assert_eq!(fixture.calls()[0]["data"], json!(id));
}

#[tokio::test]
async fn list_and_cancel_keep_original_limits_identity_and_returned_job() {
    for limit in [0, u32::MAX] {
        let fixture = Fixture::new();
        let result = list_background_jobs(&fixture, limit).await.unwrap();
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!([fixture.output()])
        );
        assert_eq!(fixture.names(), ["list"]);
        assert_eq!(fixture.calls()[0]["data"], json!(limit));
    }
    let fixture = Fixture::new();
    let id = Uuid::nil();
    let result = cancel_background_job(&fixture, id).await.unwrap();
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        json!(fixture.output())
    );
    assert_eq!(fixture.names(), ["cancel"]);
    assert_eq!(fixture.calls()[0]["data"], json!(id));
}

#[tokio::test]
async fn all_command_port_errors_keep_kind_and_message_without_starting_worker() {
    for kind in [
        PortErrorKind::Unavailable,
        PortErrorKind::NotFound,
        PortErrorKind::Conflict,
        PortErrorKind::InvalidState,
        PortErrorKind::Serialization,
        PortErrorKind::Infrastructure,
    ] {
        for command in ["enqueue", "retry", "list", "cancel"] {
            let fixture = Fixture::new();
            {
                let mut state = fixture.state.lock().unwrap();
                state.fail_at.push(command);
                state.error_kind = kind;
            }
            let error = match command {
                "enqueue" => enqueue_analysis_job(
                    &fixture,
                    serde_json::from_value(json!({"job_type": "refresh_analytics"})).unwrap(),
                )
                .await
                .unwrap_err(),
                "retry" => retry_background_job(&fixture, Uuid::nil())
                    .await
                    .unwrap_err(),
                "list" => list_background_jobs(&fixture, 0).await.unwrap_err(),
                _ => cancel_background_job(&fixture, Uuid::nil())
                    .await
                    .unwrap_err(),
            };
            assert!(
                matches!(error, ApplicationError::Port(error) if error.kind == kind && error.message == format!("injected {command}"))
            );
            tokio::task::yield_now().await;
            assert_eq!(fixture.names(), [command]);
        }
    }
}
