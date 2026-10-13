use super::{
    decide_ai_analysis_suggestion, export_ai_analysis_package,
    export_ai_analysis_response_template, import_ai_analysis_response,
    list_ai_analysis_suggestions, preview_ai_analysis_response,
};
use crate::{
    ports::{analytics::AnalyticsPort, PortError, PortErrorKind, PortResult},
    ApplicationError,
};
use async_trait::async_trait;
use football_domain::{
    AiAnalysisPackageData, AiAnalysisPackageSummary, AiAnalysisResponsePreview,
    AiAnalysisSuggestionRecord, AiSuggestionDecision, AiSuggestionDecisionDraft, AnalyticsOverview,
    AnalyticsRefreshRequest, DataQualityDecisionDraft, DataQualityFinding, DataQualitySummary,
    QueryPerformanceSummary, AI_ANALYSIS_RESPONSE_FORMAT,
};
use serde_json::{json, Value};
use std::{fs, sync::Mutex};
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

fn data() -> AiAnalysisPackageData {
    AiAnalysisPackageData {
        overview: serde_json::from_value(response()).unwrap(),
        database_summary: json!({"opaque": ["database", 1]}),
        player_review_summary: json!([{ "opaque": "player" }]),
        team_review_summary: json!([{ "opaque": "team" }]),
        ability_candidates: json!([{ "status": "pending" }]),
        schema_summary: json!([{ "table": "fixture" }]),
    }
}
fn suggestion() -> AiAnalysisSuggestionRecord {
    serde_json::from_value(json!({"id": Uuid::from_u128(1), "response_id": Uuid::from_u128(2),
        "suggestion_type": "ability_update", "title": "original", "summary": "original summary", "severity": "warning",
        "scope": {"opaque": "scope"}, "payload": {"opaque": "payload"}, "evidence": {"opaque": [1,2]},
        "status": "pending", "created_at": "2026-10-13T00:00:00.123456789Z", "decided_at": null,
        "decision_note": null, "linked_candidate_id": Uuid::from_u128(3)})).unwrap()
}
struct Fixture {
    calls: Mutex<Vec<Value>>,
    fail_at: Option<&'static str>,
    error_kind: PortErrorKind,
}
impl Fixture {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            fail_at: None,
            error_kind: PortErrorKind::Infrastructure,
        }
    }
    fn note(&self, method: &'static str, data: Value) -> PortResult<()> {
        self.calls
            .lock()
            .unwrap()
            .push(json!({"method": method, "data": data}));
        if self.fail_at == Some(method) {
            return Err(PortError::new(
                self.error_kind,
                format!("injected {method}"),
            ));
        }
        Ok(())
    }
    fn calls(&self) -> Vec<Value> {
        self.calls.lock().unwrap().clone()
    }
    fn names(&self) -> Vec<String> {
        self.calls()
            .iter()
            .map(|call| call["method"].as_str().unwrap().to_string())
            .collect()
    }
}
#[async_trait]
impl AnalyticsPort for Fixture {
    async fn build_ai_analysis_data(&self) -> PortResult<AiAnalysisPackageData> {
        self.note("build", json!({}))?;
        Ok(data())
    }
    async fn record_ai_export(&self, summary: &AiAnalysisPackageSummary) -> PortResult<()> {
        self.note("record", serde_json::to_value(summary).unwrap())
    }
    async fn import_ai_response(
        &self,
        input_path: &str,
        preview: &AiAnalysisResponsePreview,
    ) -> PortResult<Vec<AiAnalysisSuggestionRecord>> {
        self.note("import", json!({"path": input_path, "preview": preview}))?;
        Ok(vec![suggestion()])
    }
    async fn list_ai_suggestions(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<AiAnalysisSuggestionRecord>> {
        self.note("list", json!({"status": status, "limit": limit}))?;
        Ok(vec![suggestion()])
    }
    async fn decide_ai_suggestion(
        &self,
        draft: &AiSuggestionDecisionDraft,
    ) -> PortResult<AiAnalysisSuggestionRecord> {
        self.note("decide", serde_json::to_value(draft).unwrap())?;
        Ok(suggestion())
    }
    async fn overview(&self) -> PortResult<AnalyticsOverview> {
        panic!("unexpected separate overview read")
    }
    async fn refresh(&self, _request: &AnalyticsRefreshRequest) -> PortResult<AnalyticsOverview> {
        panic!("unexpected analytics refresh")
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

#[test]
fn template_writes_real_response_zip_with_optional_source_and_valid_hash() {
    let directory = tempfile::tempdir().unwrap();
    for source in [None, Some(Uuid::from_u128(99))] {
        let path = directory
            .path()
            .join("response.zip")
            .to_string_lossy()
            .to_string();
        assert_eq!(
            export_ai_analysis_response_template(path.clone(), source).unwrap(),
            path
        );
        let preview = preview_ai_analysis_response(path).unwrap();
        assert_eq!(preview.manifest.source_package_id, source);
        assert_eq!(preview.manifest.format_version, AI_ANALYSIS_RESPONSE_FORMAT);
        assert!(preview.blocking_errors.is_empty() && preview.suggestions.is_empty());
        assert!(!preview.warnings.is_empty());
    }
}

#[tokio::test]
async fn import_forwards_same_file_preview_warnings_and_unfiltered_port_result() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory
        .path()
        .join("response.zip")
        .to_string_lossy()
        .to_string();
    export_ai_analysis_response_template(path.clone(), Some(Uuid::from_u128(99))).unwrap();
    let preview = preview_ai_analysis_response(path.clone()).unwrap();
    let fixture = Fixture::new();
    let result = import_ai_analysis_response(&fixture, path.clone())
        .await
        .unwrap();
    assert_eq!(fixture.names(), ["import"]);
    assert_eq!(
        fixture.calls()[0]["data"],
        json!({"path": path, "preview": preview})
    );
    assert_eq!(serde_json::to_value(result).unwrap(), json!([suggestion()]));
}

#[tokio::test]
async fn missing_or_invalid_zip_fails_before_import_port() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("missing.zip");
    let invalid = directory.path().join("invalid.zip");
    fs::write(&invalid, b"invalid zip").unwrap();
    for path in [missing, invalid] {
        let path = path.to_string_lossy().to_string();
        assert!(matches!(
            preview_ai_analysis_response(path.clone()).unwrap_err(),
            ApplicationError::AnalysisPackage(_)
        ));
        let fixture = Fixture::new();
        assert!(matches!(
            import_ai_analysis_response(&fixture, path)
                .await
                .unwrap_err(),
            ApplicationError::AnalysisPackage(_)
        ));
        assert!(fixture.calls().is_empty());
    }
}

#[tokio::test]
async fn export_writes_package_then_records_exact_summary_with_repeatable_content_hash() {
    let directory = tempfile::tempdir().unwrap();
    let mut outputs = Vec::new();
    for name in ["first.zip", "second.zip"] {
        let fixture = Fixture::new();
        let path = directory.path().join(name).to_string_lossy().to_string();
        let summary = export_ai_analysis_package(&fixture, path.clone())
            .await
            .unwrap();
        assert_eq!(fixture.names(), ["build", "record"]);
        assert_eq!(
            fixture.calls()[1]["data"],
            serde_json::to_value(&summary).unwrap()
        );
        assert_eq!(summary.output_path, path);
        assert_eq!(summary.sample_size, 17);
        assert_eq!(summary.content_sha256.len(), 64);
        assert!(fs::read(&path).unwrap().starts_with(b"PK\x03\x04"));
        outputs.push(summary);
    }
    assert_eq!(outputs[0].content_sha256, outputs[1].content_sha256);
    assert_ne!(outputs[0].package_id, outputs[1].package_id);
}

#[tokio::test]
async fn data_read_errors_preserve_kind_and_stop_before_creating_file() {
    let directory = tempfile::tempdir().unwrap();
    for kind in kinds() {
        let mut fixture = Fixture::new();
        fixture.fail_at = Some("build");
        fixture.error_kind = kind;
        let path = directory.path().join("analysis.zip");
        assert!(
            matches!(export_ai_analysis_package(&fixture, path.to_string_lossy().to_string()).await.unwrap_err(), ApplicationError::Port(error) if error.kind == kind && error.message == "injected build")
        );
        assert_eq!(fixture.names(), ["build"]);
        assert!(!path.exists());
    }
}

#[tokio::test]
async fn file_write_failures_keep_original_error_boundary_and_skip_export_record() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().to_string_lossy().to_string();
    assert!(matches!(
        export_ai_analysis_response_template(path.clone(), None).unwrap_err(),
        ApplicationError::Io(_)
    ));
    let fixture = Fixture::new();
    assert!(matches!(
        export_ai_analysis_package(&fixture, path)
            .await
            .unwrap_err(),
        ApplicationError::AnalysisPackage(_)
    ));
    assert_eq!(fixture.names(), ["build"]);
}

#[tokio::test]
async fn metadata_errors_leave_original_written_package_and_propagate_all_port_kinds() {
    let directory = tempfile::tempdir().unwrap();
    for kind in kinds() {
        let mut fixture = Fixture::new();
        fixture.fail_at = Some("record");
        fixture.error_kind = kind;
        let path = directory.path().join("analysis.zip");
        assert!(
            matches!(export_ai_analysis_package(&fixture, path.to_string_lossy().to_string()).await.unwrap_err(), ApplicationError::Port(error) if error.kind == kind && error.message == "injected record")
        );
        assert_eq!(fixture.names(), ["build", "record"]);
        assert!(path.is_file());
        assert!(fs::metadata(&path).unwrap().len() > 0);
    }
}

#[tokio::test]
async fn import_port_errors_keep_kind_without_extra_writes_or_retry() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory
        .path()
        .join("response.zip")
        .to_string_lossy()
        .to_string();
    export_ai_analysis_response_template(path.clone(), None).unwrap();
    for kind in kinds() {
        let mut fixture = Fixture::new();
        fixture.fail_at = Some("import");
        fixture.error_kind = kind;
        assert!(
            matches!(import_ai_analysis_response(&fixture, path.clone()).await.unwrap_err(), ApplicationError::Port(error) if error.kind == kind && error.message == "injected import")
        );
        assert_eq!(fixture.names(), ["import"]);
    }
}

#[tokio::test]
async fn suggestion_list_preserves_optional_filter_limits_and_complete_records() {
    for status in [None, Some(" pending ".to_string()), Some(String::new())] {
        for limit in [0, u32::MAX] {
            let fixture = Fixture::new();
            let result = list_ai_analysis_suggestions(&fixture, status.clone(), limit)
                .await
                .unwrap();
            assert_eq!(serde_json::to_value(result).unwrap(), json!([suggestion()]));
            assert_eq!(fixture.names(), ["list"]);
            assert_eq!(
                fixture.calls()[0]["data"],
                json!({"status": status, "limit": limit})
            );
        }
    }
}

#[tokio::test]
async fn explicit_accept_and_reject_preserve_manual_identity_note_and_port_record() {
    for decision in [AiSuggestionDecision::Accept, AiSuggestionDecision::Reject] {
        for decided_by in [None, Some(String::new()), Some(" manual user ".to_string())] {
            let fixture = Fixture::new();
            let draft = AiSuggestionDecisionDraft {
                suggestion_id: Uuid::from_u128(99),
                decision,
                decided_by,
                decision_note: Some(" original note ".into()),
            };
            let expected = serde_json::to_value(&draft).unwrap();
            let result = decide_ai_analysis_suggestion(&fixture, draft)
                .await
                .unwrap();
            assert_eq!(fixture.names(), ["decide"]);
            assert_eq!(fixture.calls()[0]["data"], expected);
            assert_eq!(serde_json::to_value(result).unwrap(), json!(suggestion()));
        }
    }
}

#[tokio::test]
async fn suggestion_control_errors_preserve_all_port_kinds_without_other_actions() {
    for kind in kinds() {
        for method in ["list", "decide"] {
            let mut fixture = Fixture::new();
            fixture.fail_at = Some(method);
            fixture.error_kind = kind;
            let error = if method == "list" {
                list_ai_analysis_suggestions(&fixture, None, 0)
                    .await
                    .unwrap_err()
            } else {
                decide_ai_analysis_suggestion(
                    &fixture,
                    AiSuggestionDecisionDraft {
                        suggestion_id: Uuid::nil(),
                        decision: AiSuggestionDecision::Reject,
                        decided_by: None,
                        decision_note: None,
                    },
                )
                .await
                .unwrap_err()
            };
            assert!(
                matches!(error, ApplicationError::Port(error) if error.kind == kind && error.message == format!("injected {method}"))
            );
            assert_eq!(fixture.names(), [method]);
        }
    }
}
