use super::projection::{analysis_windows, overview_from_calculation, sample_group_ids};
use chrono::{DateTime, Utc};
use football_domain::{
    AnalyticsCalculation, AnalyticsRefreshRequest, DataQualitySummary, EvaluationSample,
    QueryPerformanceSummary, ANALYTICS_CALCULATION_VERSION,
};
use serde_json::{json, Value};
use uuid::Uuid;

fn sample(index: usize) -> EvaluationSample {
    EvaluationSample {
        review_id: Uuid::from_u128(index as u128 + 100),
        run_id: Uuid::from_u128(index as u128 + 1),
        model_version_id: Uuid::from_u128(20),
        parameter_set_id: Uuid::from_u128(30),
        model_key: "public-fixture".into(),
        model_version: "fixture".into(),
        parameter_version: "fixture".into(),
        competition_id: Some(Uuid::from_u128(40)),
        competition_name: Some("fixture".into()),
        season_id: None,
        stage_id: None,
        snapshot_type: "T-1h".into(),
        kickoff_time: format!("2026-10-13T00:00:{index:02}.123456789Z")
            .parse()
            .unwrap(),
        actual_outcome: "home_win".into(),
        home_win: 0.5,
        draw: 0.25,
        away_win: 0.25,
        log_loss: 0.5,
        brier: 0.2,
        scoreline_nll: None,
        data_coverage: 0.8,
    }
}

fn time(index: usize) -> DateTime<Utc> {
    sample(index).kickoff_time
}
fn quality() -> DataQualitySummary {
    serde_json::from_value(json!({"scan_id": Uuid::from_u128(9), "generated_at": null,
        "critical": 2, "warning": 1, "info": 0, "open_total": 3, "findings": []}))
    .unwrap()
}
fn query() -> QueryPerformanceSummary {
    serde_json::from_value(json!({"captured_at": "2026-10-13T00:00:04.123456789Z",
        "database_size_bytes": 12345, "tables": [{"schema_name": "analytics", "table_name": "fixture",
        "estimated_rows": 10, "table_size_bytes": 100, "sequential_scans": 2, "index_scans": 3,
        "dead_rows": 1, "last_analyze": null, "severity": "warning", "recommendation": "original"}]})).unwrap()
}

#[test]
fn empty_samples_keep_null_snapshot_window_and_generated_metric_window() {
    assert_eq!(
        analysis_windows(&AnalyticsRefreshRequest::default(), &[], time(9)),
        (None, None, time(9), time(9))
    );
}

#[test]
fn inferred_windows_use_min_max_without_reordering_and_keep_nanoseconds() {
    let samples = [sample(2), sample(0), sample(1)];
    assert_eq!(
        analysis_windows(&AnalyticsRefreshRequest::default(), &samples, time(9)),
        (Some(time(0)), Some(time(2)), time(0), time(2))
    );
    assert_eq!(
        samples.map(|s| s.run_id),
        [Uuid::from_u128(3), Uuid::from_u128(1), Uuid::from_u128(2)]
    );
}

#[test]
fn explicit_windows_override_samples_without_repairing_inverted_bounds() {
    let request = AnalyticsRefreshRequest {
        window_start: Some(time(8)),
        window_end: Some(time(4)),
        ..Default::default()
    };
    assert_eq!(
        analysis_windows(&request, &[sample(0), sample(2)], time(9)),
        (Some(time(8)), Some(time(4)), time(8), time(4))
    );
    assert_eq!(
        analysis_windows(&request, &[], time(9)),
        (Some(time(8)), Some(time(4)), time(8), time(4))
    );
}

#[test]
fn one_sided_windows_infer_only_missing_bound_and_fallback_only_empty_bound() {
    let start = AnalyticsRefreshRequest {
        window_start: Some(time(8)),
        ..Default::default()
    };
    let end = AnalyticsRefreshRequest {
        window_end: Some(time(4)),
        ..Default::default()
    };
    assert_eq!(
        analysis_windows(&start, &[sample(2), sample(0)], time(9)),
        (Some(time(8)), Some(time(2)), time(8), time(2))
    );
    assert_eq!(
        analysis_windows(&end, &[sample(2), sample(0)], time(9)),
        (Some(time(0)), Some(time(4)), time(0), time(4))
    );
    assert_eq!(
        analysis_windows(&start, &[], time(9)),
        (Some(time(8)), None, time(8), time(9))
    );
    assert_eq!(
        analysis_windows(&end, &[], time(9)),
        (None, Some(time(4)), time(9), time(4))
    );
}

#[test]
fn group_identity_uses_all_four_case_sensitive_strings() {
    let first = sample(0);
    let mut samples = vec![first.clone()];
    for field in 0..4 {
        let mut row = first.clone();
        row.model_version_id = Uuid::from_u128(50 + field);
        match field {
            0 => row.model_key = "Public-fixture".into(),
            1 => row.model_version = "Fixture".into(),
            2 => row.parameter_version = "Fixture".into(),
            _ => row.snapshot_type = "T-24h".into(),
        }
        samples.push(row);
    }
    let groups = sample_group_ids(&samples);
    assert_eq!(groups.len(), 5);
    for row in samples {
        assert_eq!(
            groups[&(
                row.model_key,
                row.model_version,
                row.parameter_version,
                row.snapshot_type
            )],
            (row.model_version_id, row.parameter_set_id)
        );
    }
    assert!(sample_group_ids(&[]).is_empty());
}

#[test]
fn duplicate_group_uses_last_actual_uuid_pair_in_original_input_order() {
    let first = sample(0);
    let mut last = sample(1);
    last.model_version_id = Uuid::from_u128(88);
    last.parameter_set_id = Uuid::from_u128(99);
    let key = (
        first.model_key.clone(),
        first.model_version.clone(),
        first.parameter_version.clone(),
        first.snapshot_type.clone(),
    );
    let groups = sample_group_ids(&[first.clone(), last.clone()]);
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[&key], (last.model_version_id, last.parameter_set_id));
    assert_eq!(
        sample_group_ids(&[last, first.clone()])[&key],
        (first.model_version_id, first.parameter_set_id)
    );
}

#[test]
fn missing_snapshot_keeps_empty_calculation_and_actual_diagnostics() {
    let result = serde_json::to_value(overview_from_calculation(None, quality(), query())).unwrap();
    assert_eq!(
        result,
        json!({"generated_at": null, "calculation_version": ANALYTICS_CALCULATION_VERSION,
        "sample_size": 0, "average_log_loss": null, "average_brier": null, "average_scoreline_nll": null,
        "expected_calibration_error": null, "comparisons": [], "calibration": [], "drift": [],
        "data_quality": quality(), "query_performance": query()})
    );
}

#[test]
fn saved_calculation_projects_every_metric_record_version_and_diagnostics() {
    let expected: Value = json!({"generated_at": "2026-10-13T00:00:00.123456789Z", "calculation_version": "saved-version",
        "sample_size": 17, "average_log_loss": 0.3, "average_brier": 0.2, "average_scoreline_nll": null,
        "expected_calibration_error": 0.125,
        "comparisons": [{"model_key": "fixture", "model_version": "v1", "parameter_version": "p1", "snapshot_type": "T-1h",
            "sample_size": 17, "average_log_loss": 0.3, "average_brier": 0.2, "average_scoreline_nll": null, "average_data_coverage": 0.75, "rank": 2}],
        "calibration": [{"outcome": "draw", "bucket_index": 9, "lower_bound": 0.9, "upper_bound": 1.0, "sample_size": 3,
            "predicted_mean": 0.95, "actual_rate": 1.0, "absolute_gap": 0.05, "ece_component": 0.01}],
        "drift": [{"metric_name": "log_loss", "baseline_mean": 0.5, "current_mean": 0.7, "absolute_delta": 0.2,
            "relative_delta": null, "baseline_size": 5, "current_size": 5, "severity": "warning", "direction": "up"}]});
    let calculation: AnalyticsCalculation = serde_json::from_value(expected.clone()).unwrap();
    let result = serde_json::to_value(overview_from_calculation(
        Some(calculation),
        quality(),
        query(),
    ))
    .unwrap();
    for (key, value) in expected.as_object().unwrap() {
        assert_eq!(&result[key], value, "field {key}");
    }
    assert_eq!(
        result["data_quality"],
        serde_json::to_value(quality()).unwrap()
    );
    assert_eq!(
        result["query_performance"],
        serde_json::to_value(query()).unwrap()
    );
}
