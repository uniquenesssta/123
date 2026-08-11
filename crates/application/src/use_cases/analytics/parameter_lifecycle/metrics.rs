use crate::{ApplicationError, ApplicationResult};
use football_domain::{CompetitionKind, ParameterReplayFixture, ParameterTuningCandidateRecord};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub(super) struct LifecycleSplit {
    pub(super) validation_end: usize,
}

#[derive(Debug, Clone)]
pub(super) struct LifecycleObservation {
    pub(super) actual_outcome: &'static str,
    pub(super) home_win: f64,
    pub(super) draw: f64,
    pub(super) away_win: f64,
    pub(super) scoreline_probability: Option<f64>,
}

#[derive(Debug, Clone)]
pub(super) struct LifecycleMetrics {
    pub(super) sample_count: u64,
    pub(super) average_log_loss: f64,
    pub(super) average_brier: f64,
    pub(super) average_scoreline_nll: Option<f64>,
    pub(super) expected_calibration_error: f64,
    pub(super) home_bias: f64,
    pub(super) draw_bias: f64,
    pub(super) away_bias: f64,
}

impl LifecycleMetrics {
    pub(super) fn as_json(&self) -> Value {
        json!({
            "sample_count": self.sample_count,
            "average_log_loss": self.average_log_loss,
            "average_brier": self.average_brier,
            "average_scoreline_nll": self.average_scoreline_nll,
            "expected_calibration_error": self.expected_calibration_error,
            "calibration_bias": {
                "home_win": self.home_bias,
                "draw": self.draw_bias,
                "away_win": self.away_bias,
            },
        })
    }
}

pub(super) fn lifecycle_split(
    fixtures: &[ParameterReplayFixture],
) -> ApplicationResult<LifecycleSplit> {
    if fixtures.len() < 20 {
        return Err(ApplicationError::Validation(
            "时间切分至少需要 20 场精确分区样本".to_string(),
        ));
    }
    let training_end = (fixtures.len() * 60 / 100).max(1);
    let validation_end = (fixtures.len() * 80 / 100)
        .max(training_end + 1)
        .min(fixtures.len() - 1);
    if training_end >= validation_end || validation_end >= fixtures.len() {
        return Err(ApplicationError::Validation(
            "样本无法形成训练、验证和留出窗口".to_string(),
        ));
    }
    Ok(LifecycleSplit { validation_end })
}

pub(super) fn validate_baseline_probabilities(
    fixtures: &[ParameterReplayFixture],
) -> ApplicationResult<()> {
    for fixture in fixtures {
        let probabilities = [
            fixture.baseline_home_win,
            fixture.baseline_draw,
            fixture.baseline_away_win,
        ];
        let sum = probabilities.iter().sum::<f64>();
        let valid = probabilities
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
            && sum.is_finite()
            && (sum - 1.0).abs() <= 0.0001;
        if !valid {
            return Err(ApplicationError::Validation(format!(
                "基线推演 {} 的 1X2 概率非法或未归一化，拒绝进入参数生命周期",
                fixture.run_id
            )));
        }
        if fixture
            .baseline_scoreline_probability
            .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err(ApplicationError::Validation(format!(
                "基线推演 {} 的实际比分概率非法，拒绝进入参数生命周期",
                fixture.run_id
            )));
        }
    }
    Ok(())
}

pub(super) fn baseline_metrics(fixtures: &[ParameterReplayFixture]) -> LifecycleMetrics {
    let observations = fixtures
        .iter()
        .map(|fixture| LifecycleObservation {
            actual_outcome: actual_outcome(fixture.actual_home_goals, fixture.actual_away_goals),
            home_win: fixture.baseline_home_win,
            draw: fixture.baseline_draw,
            away_win: fixture.baseline_away_win,
            scoreline_probability: fixture.baseline_scoreline_probability,
        })
        .collect::<Vec<_>>();
    lifecycle_metrics(&observations)
}

pub(super) fn lifecycle_metrics(observations: &[LifecycleObservation]) -> LifecycleMetrics {
    let mut log_loss = 0.0;
    let mut brier = 0.0;
    let mut scoreline_nll = 0.0;
    let mut scoreline_count = 0_u64;
    let mut home_bias = 0.0;
    let mut draw_bias = 0.0;
    let mut away_bias = 0.0;
    for observation in observations {
        let probabilities = [observation.home_win, observation.draw, observation.away_win];
        let targets = [
            if observation.actual_outcome == "home_win" {
                1.0
            } else {
                0.0
            },
            if observation.actual_outcome == "draw" {
                1.0
            } else {
                0.0
            },
            if observation.actual_outcome == "away_win" {
                1.0
            } else {
                0.0
            },
        ];
        let actual_probability = match observation.actual_outcome {
            "home_win" => observation.home_win,
            "draw" => observation.draw,
            _ => observation.away_win,
        }
        .clamp(1e-12, 1.0);
        log_loss += -actual_probability.ln();
        brier += probabilities
            .iter()
            .zip(targets.iter())
            .map(|(probability, target)| (probability - target).powi(2))
            .sum::<f64>();
        home_bias += observation.home_win - targets[0];
        draw_bias += observation.draw - targets[1];
        away_bias += observation.away_win - targets[2];
        if let Some(probability) = observation.scoreline_probability {
            if probability.is_finite() && probability > 0.0 {
                scoreline_nll += -probability.clamp(1e-12, 1.0).ln();
                scoreline_count += 1;
            }
        }
    }
    let count = observations.len().max(1) as f64;
    LifecycleMetrics {
        sample_count: observations.len() as u64,
        average_log_loss: round6(log_loss / count),
        average_brier: round6(brier / count),
        average_scoreline_nll: (scoreline_count > 0)
            .then(|| round6(scoreline_nll / scoreline_count as f64)),
        expected_calibration_error: round6(expected_calibration_error(observations, 10)),
        home_bias: round6(home_bias / count),
        draw_bias: round6(draw_bias / count),
        away_bias: round6(away_bias / count),
    }
}

fn expected_calibration_error(observations: &[LifecycleObservation], bucket_count: usize) -> f64 {
    if observations.is_empty() {
        return 0.0;
    }
    let mut total = 0.0;
    for outcome_index in 0..3 {
        let mut buckets = vec![Vec::<(f64, f64)>::new(); bucket_count];
        for observation in observations {
            let probability = match outcome_index {
                0 => observation.home_win,
                1 => observation.draw,
                _ => observation.away_win,
            }
            .clamp(0.0, 1.0);
            let target = match outcome_index {
                0 => observation.actual_outcome == "home_win",
                1 => observation.actual_outcome == "draw",
                _ => observation.actual_outcome == "away_win",
            };
            let index =
                ((probability * bucket_count as f64).floor() as usize).min(bucket_count - 1);
            buckets[index].push((probability, if target { 1.0 } else { 0.0 }));
        }
        for bucket in buckets {
            if bucket.is_empty() {
                continue;
            }
            let size = bucket.len() as f64;
            let predicted = bucket.iter().map(|item| item.0).sum::<f64>() / size;
            let actual = bucket.iter().map(|item| item.1).sum::<f64>() / size;
            total += (predicted - actual).abs() * size / (observations.len() * 3) as f64;
        }
    }
    total
}

pub(super) fn actual_outcome(home_goals: i16, away_goals: i16) -> &'static str {
    if home_goals > away_goals {
        "home_win"
    } else if home_goals < away_goals {
        "away_win"
    } else {
        "draw"
    }
}

fn sha256_json_value(value: &Value) -> ApplicationResult<String> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

pub(super) fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

pub(super) fn competition_kind_from_str(value: &str) -> CompetitionKind {
    match value {
        "league" => CompetitionKind::League,
        "group_stage" => CompetitionKind::GroupStage,
        "knockout_single_leg" => CompetitionKind::KnockoutSingleLeg,
        "knockout_two_leg" => CompetitionKind::KnockoutTwoLeg,
        "friendly" => CompetitionKind::Friendly,
        _ => CompetitionKind::Custom,
    }
}

pub(super) fn scoreline_probability(
    payload: &Value,
    home_goals: i16,
    away_goals: i16,
) -> Option<f64> {
    payload
        .get("scorelines")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find_map(|item| {
                let goals_a = item.get("goals_a").and_then(Value::as_i64)?;
                let goals_b = item.get("goals_b").and_then(Value::as_i64)?;
                if goals_a == i64::from(home_goals) && goals_b == i64::from(away_goals) {
                    item.get("probability").and_then(Value::as_f64)
                } else {
                    None
                }
            })
        })
}

pub(super) fn validation_key(
    candidate: &ParameterTuningCandidateRecord,
    fixtures: &[ParameterReplayFixture],
    suffix: &str,
) -> ApplicationResult<String> {
    let payload = json!({
        "candidate_id": candidate.id,
        "definition_sha256": candidate.candidate_definition_sha256,
        "partition_key": candidate.partition_key,
        "suffix": suffix,
        "run_ids": fixtures.iter().map(|item| item.run_id).collect::<Vec<_>>(),
    });
    sha256_json_value(&payload)
}

pub(super) fn required_candidate_uuid(value: Option<Uuid>, label: &str) -> ApplicationResult<Uuid> {
    value.ok_or_else(|| ApplicationError::Validation(format!("候选缺少{label}")))
}

pub(super) fn minimum_sample_size(candidate: &ParameterTuningCandidateRecord) -> u64 {
    candidate
        .constraints
        .get("minimum_sample_size")
        .and_then(Value::as_u64)
        .unwrap_or(50)
}
