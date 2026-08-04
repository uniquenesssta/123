use chrono::Utc;
use football_domain::{
    AnalyticsCalculation, CalibrationBucket, DriftFinding, EvaluationSample, ModelComparisonRow,
    ANALYTICS_CALCULATION_VERSION,
};
use std::collections::BTreeMap;

pub fn calculate_analytics(
    samples: &[EvaluationSample],
    bucket_count: u8,
    baseline_size: usize,
    current_size: usize,
) -> AnalyticsCalculation {
    let bucket_count = bucket_count.clamp(2, 20);
    let sample_size = samples.len() as u64;
    let average_log_loss = mean(samples.iter().map(|item| item.log_loss));
    let average_brier = mean(samples.iter().map(|item| item.brier));
    let average_scoreline_nll = mean(samples.iter().filter_map(|item| item.scoreline_nll));
    let calibration = calculate_calibration(samples, bucket_count);
    let expected_calibration_error = if samples.is_empty() {
        None
    } else {
        Some(round6(
            calibration.iter().map(|bucket| bucket.ece_component).sum(),
        ))
    };
    let comparisons = calculate_model_comparisons(samples);
    let drift = calculate_drift(samples, baseline_size.max(5), current_size.max(5));

    AnalyticsCalculation {
        calculation_version: ANALYTICS_CALCULATION_VERSION.to_string(),
        generated_at: Utc::now(),
        sample_size,
        average_log_loss: average_log_loss.map(round6),
        average_brier: average_brier.map(round6),
        average_scoreline_nll: average_scoreline_nll.map(round6),
        expected_calibration_error,
        calibration,
        comparisons,
        drift,
    }
}

fn calculate_calibration(samples: &[EvaluationSample], bucket_count: u8) -> Vec<CalibrationBucket> {
    let mut output = Vec::new();
    let total_binary_samples = (samples.len() * 3).max(1) as f64;
    for outcome in ["home_win", "draw", "away_win"] {
        let mut buckets: Vec<Vec<(f64, f64)>> = vec![Vec::new(); bucket_count as usize];
        for sample in samples {
            let probability = probability_for(sample, outcome).clamp(0.0, 1.0);
            let actual = if sample.actual_outcome == outcome {
                1.0
            } else {
                0.0
            };
            let index = ((probability * f64::from(bucket_count)).floor() as usize)
                .min(bucket_count as usize - 1);
            buckets[index].push((probability, actual));
        }
        for (index, values) in buckets.into_iter().enumerate() {
            if values.is_empty() {
                continue;
            }
            let size = values.len() as u64;
            let predicted_mean = values.iter().map(|item| item.0).sum::<f64>() / size as f64;
            let actual_rate = values.iter().map(|item| item.1).sum::<f64>() / size as f64;
            let gap = (predicted_mean - actual_rate).abs();
            let lower = index as f64 / f64::from(bucket_count);
            let upper = (index + 1) as f64 / f64::from(bucket_count);
            output.push(CalibrationBucket {
                outcome: outcome.to_string(),
                bucket_index: index as u8,
                lower_bound: round6(lower),
                upper_bound: round6(upper),
                sample_size: size,
                predicted_mean: round6(predicted_mean),
                actual_rate: round6(actual_rate),
                absolute_gap: round6(gap),
                ece_component: round6(gap * size as f64 / total_binary_samples),
            });
        }
    }
    output
}

fn calculate_model_comparisons(samples: &[EvaluationSample]) -> Vec<ModelComparisonRow> {
    #[derive(Default)]
    struct Aggregate {
        count: u64,
        log_loss: f64,
        brier: f64,
        scoreline_nll: f64,
        scoreline_count: u64,
        coverage: f64,
    }
    let mut groups: BTreeMap<(String, String, String, String), Aggregate> = BTreeMap::new();
    for sample in samples {
        let key = (
            sample.model_key.clone(),
            sample.model_version.clone(),
            sample.parameter_version.clone(),
            sample.snapshot_type.clone(),
        );
        let aggregate = groups.entry(key).or_default();
        aggregate.count += 1;
        aggregate.log_loss += sample.log_loss;
        aggregate.brier += sample.brier;
        aggregate.coverage += sample.data_coverage;
        if let Some(value) = sample.scoreline_nll {
            aggregate.scoreline_nll += value;
            aggregate.scoreline_count += 1;
        }
    }
    let mut rows: Vec<ModelComparisonRow> = groups
        .into_iter()
        .map(
            |((model_key, model_version, parameter_version, snapshot_type), aggregate)| {
                ModelComparisonRow {
                    model_key,
                    model_version,
                    parameter_version,
                    snapshot_type,
                    sample_size: aggregate.count,
                    average_log_loss: round6(aggregate.log_loss / aggregate.count as f64),
                    average_brier: round6(aggregate.brier / aggregate.count as f64),
                    average_scoreline_nll: (aggregate.scoreline_count > 0).then(|| {
                        round6(aggregate.scoreline_nll / aggregate.scoreline_count as f64)
                    }),
                    average_data_coverage: round6(aggregate.coverage / aggregate.count as f64),
                    rank: 0,
                }
            },
        )
        .collect();
    rows.sort_by(|left, right| {
        left.average_log_loss
            .total_cmp(&right.average_log_loss)
            .then_with(|| left.average_brier.total_cmp(&right.average_brier))
    });
    for (index, row) in rows.iter_mut().enumerate() {
        row.rank = index as u32 + 1;
    }
    rows
}

fn calculate_drift(
    samples: &[EvaluationSample],
    baseline_size: usize,
    current_size: usize,
) -> Vec<DriftFinding> {
    if samples.len() < current_size + 5 {
        return Vec::new();
    }
    let mut ordered = samples.to_vec();
    ordered.sort_by_key(|item| item.kickoff_time);
    let current_start = ordered.len().saturating_sub(current_size);
    let baseline_start = current_start.saturating_sub(baseline_size);
    let baseline = &ordered[baseline_start..current_start];
    let current = &ordered[current_start..];
    if baseline.len() < 5 || current.len() < 5 {
        return Vec::new();
    }

    [
        (
            "log_loss",
            mean(baseline.iter().map(|item| item.log_loss)),
            mean(current.iter().map(|item| item.log_loss)),
        ),
        (
            "brier",
            mean(baseline.iter().map(|item| item.brier)),
            mean(current.iter().map(|item| item.brier)),
        ),
        (
            "data_coverage",
            mean(baseline.iter().map(|item| item.data_coverage)),
            mean(current.iter().map(|item| item.data_coverage)),
        ),
    ]
    .into_iter()
    .filter_map(|(metric_name, baseline_mean, current_mean)| {
        let baseline_mean = baseline_mean?;
        let current_mean = current_mean?;
        let delta = current_mean - baseline_mean;
        let relative = if baseline_mean.abs() > 1e-12 {
            Some(delta / baseline_mean.abs())
        } else {
            None
        };
        let degradation = if metric_name == "data_coverage" {
            -delta
        } else {
            delta
        };
        let severity = if degradation >= 0.20 {
            "critical"
        } else if degradation >= 0.08 {
            "warning"
        } else {
            "stable"
        };
        Some(DriftFinding {
            metric_name: metric_name.to_string(),
            baseline_mean: round6(baseline_mean),
            current_mean: round6(current_mean),
            absolute_delta: round6(delta),
            relative_delta: relative.map(round6),
            baseline_size: baseline.len() as u64,
            current_size: current.len() as u64,
            severity: severity.to_string(),
            direction: if delta > 0.0 {
                "up"
            } else if delta < 0.0 {
                "down"
            } else {
                "flat"
            }
            .to_string(),
        })
    })
    .collect()
}

fn probability_for(sample: &EvaluationSample, outcome: &str) -> f64 {
    match outcome {
        "home_win" => sample.home_win,
        "draw" => sample.draw,
        _ => sample.away_win,
    }
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut total = 0.0;
    let mut count = 0_u64;
    for value in values {
        if value.is_finite() {
            total += value;
            count += 1;
        }
    }
    (count > 0).then(|| total / count as f64)
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use uuid::Uuid;

    fn sample(index: i64, probability: f64, actual: &str) -> EvaluationSample {
        EvaluationSample {
            review_id: Uuid::new_v4(),
            run_id: Uuid::new_v4(),
            model_version_id: Uuid::new_v4(),
            parameter_set_id: Uuid::new_v4(),
            model_key: "p7".to_string(),
            model_version: "1".to_string(),
            parameter_version: "1".to_string(),
            competition_id: None,
            competition_name: None,
            season_id: None,
            stage_id: None,
            snapshot_type: "T-1h".to_string(),
            kickoff_time: Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
                + Duration::days(index),
            actual_outcome: actual.to_string(),
            home_win: probability,
            draw: (1.0 - probability) / 2.0,
            away_win: (1.0 - probability) / 2.0,
            log_loss: if actual == "home_win" {
                -probability.ln()
            } else {
                -((1.0 - probability) / 2.0).ln()
            },
            brier: 0.2,
            scoreline_nll: Some(2.0),
            data_coverage: 1.0,
        }
    }

    #[test]
    fn produces_comparison_and_calibration() {
        let samples = (0..30)
            .map(|index| sample(index, 0.6, if index % 2 == 0 { "home_win" } else { "draw" }))
            .collect::<Vec<_>>();
        let result = calculate_analytics(&samples, 10, 10, 10);
        assert_eq!(result.sample_size, 30);
        assert!(!result.calibration.is_empty());
        assert_eq!(result.comparisons.len(), 1);
    }
}
