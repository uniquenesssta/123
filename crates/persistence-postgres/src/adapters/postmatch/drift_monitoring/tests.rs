use super::calculation::{drift_status, drift_windows, prepare_drift, window_json};
use crate::sha256_json;
use football_domain::{DriftFinding, EvaluationSample, POSTMATCH_MONITORING_VERSION};
use serde_json::json;
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

fn finding(severity: &str) -> DriftFinding {
    DriftFinding {
        metric_name: "log_loss".into(),
        baseline_mean: 0.5,
        current_mean: 0.7,
        absolute_delta: 0.2,
        relative_delta: None,
        baseline_size: 5,
        current_size: 5,
        severity: severity.into(),
        direction: "up".into(),
    }
}

#[test]
fn insufficient_samples_override_all_findings() {
    for count in [0, 4, 9] {
        assert_eq!(
            drift_status(count, 5, 5, &[finding("critical"), finding("warning")]),
            "insufficient"
        );
    }
}

#[test]
fn complete_windows_prioritize_critical_independent_of_finding_order() {
    for findings in [
        [finding("warning"), finding("critical")],
        [finding("critical"), finding("warning")],
    ] {
        assert_eq!(drift_status(10, 5, 5, &findings), "critical");
    }
}

#[test]
fn warning_and_stable_keep_original_case_sensitive_policy() {
    assert_eq!(drift_status(10, 5, 5, &[finding("warning")]), "warning");
    assert_eq!(drift_status(10, 5, 5, &[]), "stable");
    assert_eq!(
        drift_status(
            10,
            5,
            5,
            &[finding("Critical"), finding("Warning"), finding("unknown")]
        ),
        "stable"
    );
}

#[test]
fn window_slices_saturate_for_empty_and_short_histories() {
    let samples: Vec<_> = (0..3).map(sample).collect();
    let (baseline, current) = drift_windows(&[], 5, 5);
    assert!(baseline.is_empty() && current.is_empty());
    let (baseline, current) = drift_windows(&samples, 5, 5);
    assert!(baseline.is_empty());
    assert_eq!(current.len(), 3);
    assert_eq!(current[0].run_id, samples[0].run_id);
    let samples: Vec<_> = (0..7).map(sample).collect();
    let (baseline, current) = drift_windows(&samples, 5, 5);
    assert_eq!((baseline.len(), current.len()), (2, 5));
    assert_eq!(baseline[1].run_id, samples[1].run_id);
    assert_eq!(current[0].run_id, samples[2].run_id);
}

#[test]
fn window_slices_use_latest_disjoint_windows_and_drop_old_surplus() {
    let samples: Vec<_> = (0..13).map(sample).collect();
    let (baseline, current) = drift_windows(&samples, 5, 5);
    assert_eq!((baseline.len(), current.len()), (5, 5));
    assert_eq!(baseline[0].run_id, samples[3].run_id);
    assert_eq!(baseline[4].run_id, samples[7].run_id);
    assert_eq!(current[0].run_id, samples[8].run_id);
    assert_eq!(current[4].run_id, samples[12].run_id);
}

#[test]
fn window_json_preserves_input_order_endpoints_and_nanoseconds() {
    let samples = [sample(2), sample(0), sample(1)];
    assert_eq!(
        window_json(&samples),
        json!({
            "sample_size": 3,
            "start": "2026-10-13T00:00:02.123456789Z",
            "end": "2026-10-13T00:00:01.123456789Z",
            "run_ids": [samples[0].run_id, samples[1].run_id, samples[2].run_id]
        })
    );
    assert_eq!(
        window_json(&[]),
        json!({"sample_size": 0, "start": null, "end": null, "run_ids": []})
    );
}

#[test]
fn prepared_empty_run_keeps_requested_identity_and_actual_empty_windows() {
    let prepared = prepare_drift(&[], Vec::new(), "partition", 5, 5).unwrap();
    assert_eq!(prepared.status, "insufficient");
    assert_eq!((prepared.baseline_count, prepared.current_count), (0, 0));
    assert_eq!(prepared.calculation.sample_size, 0);
    assert!(prepared.calculation.drift.is_empty());
    assert_eq!(prepared.baseline_window, window_json(&[]));
    assert_eq!(prepared.current_window, window_json(&[]));
    assert_eq!(
        prepared.run_key,
        sha256_json(&json!({
            "partition_key": "partition", "baseline_size": 5, "current_size": 5,
            "settlement_ids": [], "calculation_version": POSTMATCH_MONITORING_VERSION
        }))
        .unwrap()
    );
}

#[test]
fn run_identity_is_repeatable_and_sensitive_to_partition_requested_sizes_and_order() {
    let samples = [sample(0), sample(1)];
    let ids = vec![Uuid::from_u128(50), Uuid::from_u128(51)];
    let first = prepare_drift(&samples, ids.clone(), "partition", 5, 5).unwrap();
    assert_eq!(
        first.run_key,
        prepare_drift(&samples, ids.clone(), "partition", 5, 5)
            .unwrap()
            .run_key
    );
    assert_ne!(
        first.run_key,
        prepare_drift(&samples, ids.clone(), "other", 5, 5)
            .unwrap()
            .run_key
    );
    assert_ne!(
        first.run_key,
        prepare_drift(&samples, ids.clone(), "partition", 6, 5)
            .unwrap()
            .run_key
    );
    assert_ne!(
        first.run_key,
        prepare_drift(&samples, ids.clone(), "partition", 5, 6)
            .unwrap()
            .run_key
    );
    let reversed = ids.into_iter().rev().collect();
    assert_ne!(
        first.run_key,
        prepare_drift(&samples, reversed, "partition", 5, 5)
            .unwrap()
            .run_key
    );
    assert_eq!((first.baseline_count, first.current_count), (0, 2));
}
