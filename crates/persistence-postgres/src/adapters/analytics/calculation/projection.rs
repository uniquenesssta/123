use chrono::{DateTime, Utc};
use football_domain::{
    AnalyticsCalculation, AnalyticsOverview, AnalyticsRefreshRequest, DataQualitySummary,
    EvaluationSample, QueryPerformanceSummary, ANALYTICS_CALCULATION_VERSION,
};
use std::collections::HashMap;
use uuid::Uuid;

pub(super) fn analysis_windows(
    request: &AnalyticsRefreshRequest,
    samples: &[EvaluationSample],
    generated_at: DateTime<Utc>,
) -> (
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    DateTime<Utc>,
    DateTime<Utc>,
) {
    let window_start = request
        .window_start
        .or_else(|| samples.iter().map(|item| item.kickoff_time).min());
    let window_end = request
        .window_end
        .or_else(|| samples.iter().map(|item| item.kickoff_time).max());
    let metric_window_start = window_start.unwrap_or(generated_at);
    let metric_window_end = window_end.unwrap_or(generated_at);
    (
        window_start,
        window_end,
        metric_window_start,
        metric_window_end,
    )
}

pub(super) fn sample_group_ids(
    samples: &[EvaluationSample],
) -> HashMap<(String, String, String, String), (Uuid, Uuid)> {
    samples
        .iter()
        .map(|sample| {
            (
                (
                    sample.model_key.clone(),
                    sample.model_version.clone(),
                    sample.parameter_version.clone(),
                    sample.snapshot_type.clone(),
                ),
                (sample.model_version_id, sample.parameter_set_id),
            )
        })
        .collect()
}

pub(super) fn overview_from_calculation(
    calculation: Option<AnalyticsCalculation>,
    quality: DataQualitySummary,
    query_performance: QueryPerformanceSummary,
) -> AnalyticsOverview {
    if let Some(calculation) = calculation {
        AnalyticsOverview {
            generated_at: Some(calculation.generated_at),
            calculation_version: calculation.calculation_version,
            sample_size: calculation.sample_size,
            average_log_loss: calculation.average_log_loss,
            average_brier: calculation.average_brier,
            average_scoreline_nll: calculation.average_scoreline_nll,
            expected_calibration_error: calculation.expected_calibration_error,
            comparisons: calculation.comparisons,
            calibration: calculation.calibration,
            drift: calculation.drift,
            data_quality: quality,
            query_performance,
        }
    } else {
        AnalyticsOverview {
            generated_at: None,
            calculation_version: ANALYTICS_CALCULATION_VERSION.to_string(),
            sample_size: 0,
            average_log_loss: None,
            average_brier: None,
            average_scoreline_nll: None,
            expected_calibration_error: None,
            comparisons: Vec::new(),
            calibration: Vec::new(),
            drift: Vec::new(),
            data_quality: quality,
            query_performance,
        }
    }
}
