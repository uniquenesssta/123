use super::{analytics_overview, refresh_analytics};
use crate::{
    ports::{analytics::AnalyticsPort, PortError, PortErrorKind, PortResult},
    ApplicationError,
};
use async_trait::async_trait;
use football_domain::{
    AiAnalysisPackageData, AiAnalysisPackageSummary, AiAnalysisResponsePreview,
    AiAnalysisSuggestionRecord, AiSuggestionDecisionDraft, AnalyticsOverview,
    AnalyticsRefreshRequest, DataQualityDecisionDraft, DataQualityFinding, DataQualitySummary,
    QueryPerformanceSummary,
};
use serde_json::{json, Value};
use std::sync::Mutex;
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

struct Fixture {
    calls: Mutex<Vec<&'static str>>,
    request: Mutex<Option<AnalyticsRefreshRequest>>,
    error: Option<PortError>,
}
impl Fixture {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            request: Mutex::new(None),
            error: None,
        }
    }
    fn result(&self) -> PortResult<AnalyticsOverview> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        Ok(serde_json::from_value(response()).unwrap())
    }
}
#[async_trait]
impl AnalyticsPort for Fixture {
    async fn overview(&self) -> PortResult<AnalyticsOverview> {
        self.calls.lock().unwrap().push("overview");
        self.result()
    }
    async fn refresh(&self, request: &AnalyticsRefreshRequest) -> PortResult<AnalyticsOverview> {
        self.calls.lock().unwrap().push("refresh");
        *self.request.lock().unwrap() = Some(request.clone());
        self.result()
    }
    async fn run_data_quality_scan(&self) -> PortResult<DataQualitySummary> {
        panic!("unexpected quality scan")
    }
    async fn decide_data_quality(
        &self,
        _draft: &DataQualityDecisionDraft,
    ) -> PortResult<DataQualityFinding> {
        panic!("unexpected quality decision")
    }
    async fn capture_query_performance(&self) -> PortResult<QueryPerformanceSummary> {
        panic!("unexpected query scan")
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

#[tokio::test]
async fn refresh_preserves_every_request_field_and_full_overview() {
    let fixture = Fixture::new();
    let request = AnalyticsRefreshRequest {
        competition_id: Some(Uuid::from_u128(99)),
        window_start: Some("2026-10-14T00:00:00.123456789Z".parse().unwrap()),
        window_end: Some("2026-10-12T00:00:00.987654321Z".parse().unwrap()),
        bucket_count: 0,
        baseline_size: 0,
        current_size: usize::MAX,
    };
    let result = refresh_analytics(&fixture, &request).await.unwrap();
    assert_eq!(
        serde_json::to_value(fixture.request.lock().unwrap().as_ref().unwrap()).unwrap(),
        serde_json::to_value(request).unwrap()
    );
    assert_eq!(serde_json::to_value(result).unwrap(), response());
    assert_eq!(*fixture.calls.lock().unwrap(), ["refresh"]);
}

#[tokio::test]
async fn refresh_preserves_default_unbounded_request_without_extra_calls() {
    let fixture = Fixture::new();
    let request = AnalyticsRefreshRequest::default();
    refresh_analytics(&fixture, &request).await.unwrap();
    assert_eq!(
        serde_json::to_value(fixture.request.lock().unwrap().as_ref().unwrap()).unwrap(),
        serde_json::to_value(request).unwrap()
    );
    assert_eq!(*fixture.calls.lock().unwrap(), ["refresh"]);
}

#[tokio::test]
async fn overview_keeps_snapshot_diagnostics_and_does_not_refresh() {
    let fixture = Fixture::new();
    assert_eq!(
        serde_json::to_value(analytics_overview(&fixture).await.unwrap()).unwrap(),
        response()
    );
    assert_eq!(*fixture.calls.lock().unwrap(), ["overview"]);
    assert!(fixture.request.lock().unwrap().is_none());
}

#[tokio::test]
async fn refresh_propagates_all_port_errors_without_retry_or_followup_read() {
    for kind in [
        PortErrorKind::Unavailable,
        PortErrorKind::NotFound,
        PortErrorKind::Conflict,
        PortErrorKind::InvalidState,
        PortErrorKind::Serialization,
        PortErrorKind::Infrastructure,
    ] {
        let mut fixture = Fixture::new();
        fixture.error = Some(PortError::new(kind, "original refresh failure"));
        assert!(
            matches!(refresh_analytics(&fixture, &AnalyticsRefreshRequest::default()).await.unwrap_err(), ApplicationError::Port(error) if error.kind == kind && error.message == "original refresh failure")
        );
        assert_eq!(*fixture.calls.lock().unwrap(), ["refresh"]);
    }
}

#[tokio::test]
async fn overview_propagates_all_port_errors_without_refresh_or_retry() {
    for kind in [
        PortErrorKind::Unavailable,
        PortErrorKind::NotFound,
        PortErrorKind::Conflict,
        PortErrorKind::InvalidState,
        PortErrorKind::Serialization,
        PortErrorKind::Infrastructure,
    ] {
        let mut fixture = Fixture::new();
        fixture.error = Some(PortError::new(kind, "original overview failure"));
        assert!(
            matches!(analytics_overview(&fixture).await.unwrap_err(), ApplicationError::Port(error) if error.kind == kind && error.message == "original overview failure")
        );
        assert_eq!(*fixture.calls.lock().unwrap(), ["overview"]);
        assert!(fixture.request.lock().unwrap().is_none());
    }
}
