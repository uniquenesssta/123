use super::{postmatch_overview, refresh_postmatch_monitoring};
use crate::{
    ports::{postmatch::PostmatchMonitoringPort, PortError, PortErrorKind, PortResult},
    ApplicationError,
};
use async_trait::async_trait;
use football_domain::{PostmatchMonitoringRequest, PostmatchOverview};
use serde_json::{json, Value};
use std::sync::Mutex;
use uuid::Uuid;

fn response() -> Value {
    json!({
        "settlement_count": 17, "pending_evidence_count": 2, "scored_evidence_count": 5,
        "settlements": [], "evidence_queue": [],
        "provider_scores": [{
            "id": Uuid::from_u128(1), "provider_id": Uuid::from_u128(2), "provider_name": "fixture",
            "scope_key": "original-scope", "competition_id": Uuid::from_u128(3),
            "competition_profile_id": Uuid::from_u128(4), "model_version_id": Uuid::from_u128(5),
            "parameter_set_id": Uuid::from_u128(6), "horizon": "T-1h", "sample_size": 3,
            "correct_count": 1, "partial_count": 1, "incorrect_count": 0, "not_verifiable_count": 1,
            "accuracy_mean": 0.75, "timeliness_mean": 0.5, "reliability_mean": 0.6,
            "weighted_score": 0.675, "decision_set_sha256": "original-digest",
            "calculation_version": "original-version", "generated_at": "2026-10-13T00:00:00.123456789Z"
        }],
        "drift_runs": [{
            "id": Uuid::from_u128(7), "competition_id": Uuid::from_u128(3), "competition_name": "fixture",
            "competition_profile_id": Uuid::from_u128(4), "model_version_id": Uuid::from_u128(5),
            "model_version": "fixture", "parameter_set_id": Uuid::from_u128(6), "parameter_version": "fixture",
            "horizon": "T-1h", "partition_key": "original-partition", "baseline_size": 5, "current_size": 5,
            "baseline_window": {"opaque": "baseline"}, "current_window": {"opaque": "current"},
            "status": "warning", "run_key": "original-run", "calculation_version": "original-version",
            "findings": [{"metric_name": "original-metric", "baseline_mean": 0.5, "current_mean": 0.7,
                "absolute_delta": 0.2, "relative_delta": null, "severity": "warning", "direction": "up"}],
            "generated_at": "2026-10-13T00:00:00.123456789Z"
        }]
    })
}

struct Fixture {
    calls: Mutex<Vec<&'static str>>,
    request: Mutex<Option<PostmatchMonitoringRequest>>,
    limit: Mutex<Option<u32>>,
    output: Value,
    error: Option<PortError>,
}

impl Fixture {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            request: Mutex::new(None),
            limit: Mutex::new(None),
            output: response(),
            error: None,
        }
    }

    fn result(&self) -> PortResult<PostmatchOverview> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        Ok(serde_json::from_value(self.output.clone()).unwrap())
    }
}

#[async_trait]
impl PostmatchMonitoringPort for Fixture {
    async fn refresh_monitoring(
        &self,
        request: &PostmatchMonitoringRequest,
    ) -> PortResult<PostmatchOverview> {
        self.calls.lock().unwrap().push("refresh");
        *self.request.lock().unwrap() = Some(request.clone());
        self.result()
    }

    async fn overview(&self, limit: u32) -> PortResult<PostmatchOverview> {
        self.calls.lock().unwrap().push("overview");
        *self.limit.lock().unwrap() = Some(limit);
        self.result()
    }
}

fn request(horizon: &str, baseline_size: usize, current_size: usize) -> PostmatchMonitoringRequest {
    PostmatchMonitoringRequest {
        competition_id: Uuid::from_u128(90),
        horizon: horizon.into(),
        baseline_size,
        current_size,
    }
}

#[tokio::test]
async fn refresh_preserves_request_and_complete_provider_drift_response() {
    let fixture = Fixture::new();
    let input = request("T-1h", 100, 50);
    let expected = serde_json::to_value(&input).unwrap();
    let result = refresh_postmatch_monitoring(&fixture, input).await.unwrap();
    assert_eq!(
        serde_json::to_value(fixture.request.lock().unwrap().as_ref().unwrap()).unwrap(),
        expected
    );
    assert_eq!(serde_json::to_value(result).unwrap(), response());
    assert_eq!(*fixture.calls.lock().unwrap(), ["refresh"]);
    assert!(fixture.limit.lock().unwrap().is_none());
}

#[tokio::test]
async fn refresh_leaves_horizon_and_window_validation_at_original_port_boundary() {
    for (horizon, baseline, current) in [
        (" T-1h ", 0, 4),
        ("T-90m", usize::MAX, usize::MAX),
        ("", 5, 5),
    ] {
        let fixture = Fixture::new();
        let input = request(horizon, baseline, current);
        let expected = serde_json::to_value(&input).unwrap();
        refresh_postmatch_monitoring(&fixture, input).await.unwrap();
        assert_eq!(
            serde_json::to_value(fixture.request.lock().unwrap().as_ref().unwrap()).unwrap(),
            expected
        );
        assert_eq!(*fixture.calls.lock().unwrap(), ["refresh"]);
    }
}

#[tokio::test]
async fn overview_preserves_zero_and_max_limit_and_complete_response() {
    for limit in [0, u32::MAX] {
        let fixture = Fixture::new();
        let result = postmatch_overview(&fixture, limit).await.unwrap();
        assert_eq!(*fixture.limit.lock().unwrap(), Some(limit));
        assert_eq!(serde_json::to_value(result).unwrap(), response());
        assert_eq!(*fixture.calls.lock().unwrap(), ["overview"]);
        assert!(fixture.request.lock().unwrap().is_none());
    }
}

#[tokio::test]
async fn refresh_propagates_all_port_errors_without_retry_or_overview() {
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
            matches!(refresh_postmatch_monitoring(&fixture, request("T-1h", 5, 5)).await.unwrap_err(), ApplicationError::Port(error) if error.kind == kind && error.message == "original refresh failure")
        );
        assert_eq!(*fixture.calls.lock().unwrap(), ["refresh"]);
        assert!(fixture.limit.lock().unwrap().is_none());
    }
}

#[tokio::test]
async fn overview_propagates_all_port_errors_without_refresh() {
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
            matches!(postmatch_overview(&fixture, 0).await.unwrap_err(), ApplicationError::Port(error) if error.kind == kind && error.message == "original overview failure")
        );
        assert_eq!(*fixture.calls.lock().unwrap(), ["overview"]);
        assert!(fixture.request.lock().unwrap().is_none());
    }
}

#[tokio::test]
async fn empty_overview_keeps_returned_counts_without_inventing_records() {
    let mut fixture = Fixture::new();
    fixture.output = json!({"settlement_count": 0, "pending_evidence_count": 0, "scored_evidence_count": 0,
        "settlements": [], "evidence_queue": [], "provider_scores": [], "drift_runs": []});
    let result = postmatch_overview(&fixture, 0).await.unwrap();
    assert_eq!(serde_json::to_value(result).unwrap(), fixture.output);
    assert_eq!(*fixture.calls.lock().unwrap(), ["overview"]);
}
