use crate::{sha256_json, PersistenceResult};
use football_analytics_engine::calculate_analytics;
use football_domain::{
    AnalyticsCalculation, DriftFinding, EvaluationSample, POSTMATCH_MONITORING_VERSION,
};
use serde_json::{json, Value};
use uuid::Uuid;

pub(super) struct PreparedDrift {
    pub(super) calculation: AnalyticsCalculation,
    pub(super) status: &'static str,
    pub(super) baseline_count: usize,
    pub(super) current_count: usize,
    pub(super) baseline_window: Value,
    pub(super) current_window: Value,
    pub(super) run_key: String,
}

pub(super) fn prepare_drift(
    samples: &[EvaluationSample],
    settlement_ids: Vec<Uuid>,
    partition_key: &str,
    baseline_size: usize,
    current_size: usize,
) -> PersistenceResult<PreparedDrift> {
    let calculation = calculate_analytics(samples, 10, baseline_size, current_size);
    let status = drift_status(
        samples.len(),
        baseline_size,
        current_size,
        &calculation.drift,
    );
    let (baseline_slice, current_slice) = drift_windows(samples, baseline_size, current_size);
    let baseline_window = window_json(baseline_slice);
    let current_window = window_json(current_slice);
    let run_payload = json!({
        "partition_key": partition_key,
        "baseline_size": baseline_size,
        "current_size": current_size,
        "settlement_ids": settlement_ids,
        "calculation_version": POSTMATCH_MONITORING_VERSION,
    });
    let run_key = sha256_json(&run_payload)?;
    Ok(PreparedDrift {
        calculation,
        status,
        baseline_count: baseline_slice.len(),
        current_count: current_slice.len(),
        baseline_window,
        current_window,
        run_key,
    })
}

pub(super) fn drift_status(
    sample_count: usize,
    baseline_size: usize,
    current_size: usize,
    findings: &[DriftFinding],
) -> &'static str {
    let enough = sample_count >= baseline_size + current_size;
    if !enough {
        "insufficient"
    } else if findings.iter().any(|item| item.severity == "critical") {
        "critical"
    } else if findings.iter().any(|item| item.severity == "warning") {
        "warning"
    } else {
        "stable"
    }
}

pub(super) fn drift_windows(
    samples: &[EvaluationSample],
    baseline_size: usize,
    current_size: usize,
) -> (&[EvaluationSample], &[EvaluationSample]) {
    let current_start = samples.len().saturating_sub(current_size);
    let baseline_start = current_start.saturating_sub(baseline_size);
    let baseline_slice = &samples[baseline_start..current_start];
    let current_slice = &samples[current_start..];
    (baseline_slice, current_slice)
}

pub(super) fn window_json(samples: &[EvaluationSample]) -> Value {
    json!({
        "sample_size": samples.len(),
        "start": samples.first().map(|item| item.kickoff_time),
        "end": samples.last().map(|item| item.kickoff_time),
        "run_ids": samples.iter().map(|item| item.run_id).collect::<Vec<_>>(),
    })
}
