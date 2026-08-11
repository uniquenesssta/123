use super::metrics::{
    actual_outcome, baseline_metrics, competition_kind_from_str, lifecycle_metrics,
    lifecycle_split, minimum_sample_size, required_candidate_uuid, round6, scoreline_probability,
    validate_baseline_probabilities, validation_key, LifecycleObservation,
};
use crate::{
    model_registry::ModelRegistry, ports::analytics::ParameterLifecyclePort, ApplicationError,
    ApplicationResult,
};
use football_domain::{
    MatchContext, ModelIdentity, ParameterLifecycleReadinessRequest,
    ParameterShadowValidationRecord, ParameterShadowValidationRequest,
};
use football_model_api::ModelRequest;
use serde_json::json;
use uuid::Uuid;

pub(crate) async fn execute<P>(
    port: &P,
    registry: &ModelRegistry,
    request: ParameterShadowValidationRequest,
) -> ApplicationResult<ParameterShadowValidationRecord>
where
    P: ParameterLifecyclePort + ?Sized,
{
    let candidate = port.read_tuning_candidate(request.candidate_id).await?;
    if !matches!(
        candidate.status.as_str(),
        "accepted_for_backtest" | "blocked_by_h" | "shadow_failed"
    ) {
        return Err(ApplicationError::Validation(
            "候选必须先由用户确认进入影子验证队列".to_string(),
        ));
    }
    let competition_id = required_candidate_uuid(candidate.competition_id, "赛事范围")?;
    let competition_profile_id =
        required_candidate_uuid(candidate.competition_profile_id, "赛事 Profile")?;
    let baseline_model_version_id =
        required_candidate_uuid(candidate.baseline_model_version_id, "基线模型版本")?;
    let baseline_parameter_set_id =
        required_candidate_uuid(candidate.baseline_parameter_set_id, "基线参数版本")?;
    let candidate_parameter_set_id =
        required_candidate_uuid(candidate.candidate_parameter_set_id, "候选参数版本")?;
    let readiness_request = ParameterLifecycleReadinessRequest {
        competition_id: Some(competition_id),
        snapshot_type: candidate.snapshot_type.clone(),
        minimum_sample_size: minimum_sample_size(&candidate),
    };
    let readiness = port.readiness(&readiness_request).await?;
    if !readiness.ready_for_shadow_validation {
        let key = validation_key(&candidate, &[], "blocked_by_h")?;
        let record = ParameterShadowValidationRecord {
            id: Uuid::new_v4(),
            candidate_id: candidate.id,
            validation_key: key,
            partition_key: readiness.partition_key,
            sample_count: readiness.eligible_sample_count,
            baseline_metrics: candidate.baseline_metrics.clone(),
            candidate_metrics: json!({}),
            metric_deltas: json!({}),
            gate_results: json!({
                "passed": false,
                "h_contract_ready": readiness.h_contract_ready,
                "blocked_reasons": readiness.blocked_reasons,
                "automatic_promotion": false,
            }),
            status: "blocked".to_string(),
            generated_at: chrono::Utc::now(),
        };
        return Ok(port.save_shadow_validation(&record).await?);
    }

    let mut fixtures = port
        .load_replay_fixtures(
            competition_id,
            competition_profile_id,
            &candidate.snapshot_type,
            baseline_model_version_id,
            baseline_parameter_set_id,
        )
        .await?;
    fixtures.sort_by_key(|item| (item.kickoff_time, item.run_id));
    validate_baseline_probabilities(&fixtures)?;
    if fixtures.len() < minimum_sample_size(&candidate) as usize {
        return Err(ApplicationError::Validation(
            "精确分区样本在验证前发生变化，已拒绝继续".to_string(),
        ));
    }
    let split = lifecycle_split(&fixtures)?;
    let holdout = &fixtures[split.validation_end..];
    let candidate_parameters = port
        .read_parameter_set_definition(candidate_parameter_set_id)
        .await?;
    let model = registry
        .get(&candidate.model_key)
        .ok_or_else(|| ApplicationError::ModelNotFound(candidate.model_key.clone()))?;
    model
        .validate_parameters(candidate_parameters.as_value())
        .map_err(|error| ApplicationError::Model(error.to_string()))?;
    let candidate_model_version = candidate
        .candidate_model_version
        .clone()
        .ok_or_else(|| ApplicationError::Validation("候选模型版本为空".to_string()))?;
    let candidate_parameter_version = candidate
        .candidate_parameter_version
        .clone()
        .ok_or_else(|| ApplicationError::Validation("候选参数版本为空".to_string()))?;
    let mut observations = Vec::with_capacity(holdout.len());
    let mut finite_probabilities = true;
    for fixture in holdout {
        let context = MatchContext {
            match_key: fixture.match_key.clone(),
            kickoff_time: fixture.kickoff_time,
            competition_id: fixture.competition_id,
            season_id: fixture.season_id,
            stage_id: fixture.stage_id,
            competition_kind: competition_kind_from_str(&fixture.competition_kind),
            home_team_name: fixture.home_team_name.clone(),
            away_team_name: fixture.away_team_name.clone(),
            metadata: json!({
                "parameter_lifecycle": "shadow_replay",
                "source_run_id": fixture.run_id,
            }),
        };
        if !model.supports(&context) {
            return Err(ApplicationError::Model(format!(
                "模型 {} 不支持留出样本的赛事类型 {}",
                candidate.model_key, fixture.competition_kind
            )));
        }
        let output = model
            .predict(&ModelRequest {
                context,
                identity: ModelIdentity {
                    model_id: candidate.model_key.clone(),
                    model_version: candidate_model_version.clone(),
                    parameter_version: candidate_parameter_version.clone(),
                    rule_package_version: fixture.rule_package_version.clone(),
                },
                snapshot_type: fixture.snapshot_type.clone(),
                input: fixture.input_payload.clone(),
                parameters: candidate_parameters.as_value().clone(),
            })
            .map_err(|error| ApplicationError::Model(error.to_string()))?;
        let probabilities = [
            output.summary.home_win,
            output.summary.draw,
            output.summary.away_win,
        ];
        let sum = probabilities.iter().sum::<f64>();
        let probability_vector_is_valid = probabilities
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
            && sum.is_finite()
            && (sum - 1.0).abs() <= 0.0001;
        finite_probabilities &= probability_vector_is_valid;
        let safe_probabilities = if probability_vector_is_valid {
            probabilities
        } else {
            [0.0, 0.0, 0.0]
        };
        let actual_scoreline_probability = scoreline_probability(
            &output.payload,
            fixture.actual_home_goals,
            fixture.actual_away_goals,
        )
        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value));
        observations.push(LifecycleObservation {
            actual_outcome: actual_outcome(fixture.actual_home_goals, fixture.actual_away_goals),
            home_win: safe_probabilities[0],
            draw: safe_probabilities[1],
            away_win: safe_probabilities[2],
            scoreline_probability: actual_scoreline_probability,
        });
    }
    let baseline = baseline_metrics(holdout);
    let challenger = lifecycle_metrics(&observations);
    let log_loss_delta = challenger.average_log_loss - baseline.average_log_loss;
    let brier_delta = challenger.average_brier - baseline.average_brier;
    let ece_delta = challenger.expected_calibration_error - baseline.expected_calibration_error;
    let scoreline_delta = match (
        challenger.average_scoreline_nll,
        baseline.average_scoreline_nll,
    ) {
        (Some(left), Some(right)) => Some(left - right),
        _ => None,
    };
    let log_loss_pass = log_loss_delta <= 0.001;
    let brier_pass = brier_delta <= 0.001;
    let ece_pass = ece_delta <= 0.005;
    let scoreline_pass = scoreline_delta.is_none_or(|value| value <= 0.01);
    let binding_unchanged = readiness.active_model_version_id
        == candidate.baseline_model_version_id
        && readiness.active_parameter_set_id == candidate.baseline_parameter_set_id;
    let passed = finite_probabilities
        && binding_unchanged
        && log_loss_pass
        && brier_pass
        && ece_pass
        && scoreline_pass;
    let key = validation_key(&candidate, holdout, "holdout")?;
    let record = ParameterShadowValidationRecord {
        id: Uuid::new_v4(),
        candidate_id: candidate.id,
        validation_key: key,
        partition_key: readiness.partition_key,
        sample_count: holdout.len() as u64,
        baseline_metrics: baseline.as_json(),
        candidate_metrics: challenger.as_json(),
        metric_deltas: json!({
            "average_log_loss": round6(log_loss_delta),
            "average_brier": round6(brier_delta),
            "expected_calibration_error": round6(ece_delta),
            "average_scoreline_nll": scoreline_delta.map(round6),
        }),
        gate_results: json!({
            "passed": passed,
            "h_contract_ready": readiness.h_contract_ready,
            "finite_probabilities": finite_probabilities,
            "holdout_log_loss_not_worse": log_loss_pass,
            "holdout_brier_not_worse": brier_pass,
            "holdout_ece_not_worse": ece_pass,
            "scoreline_nll_no_material_regression": scoreline_pass,
            "binding_unchanged": binding_unchanged,
            "automatic_promotion": false,
            "provider_state": "NOT_BUNDLED",
        }),
        status: if passed { "passed" } else { "failed" }.to_string(),
        generated_at: chrono::Utc::now(),
    };
    Ok(port.save_shadow_validation(&record).await?)
}
