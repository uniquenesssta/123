use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_analysis_package::{
    read_analysis_response, response_template_bytes, write_analysis_package,
};
use football_domain::{
    AiAnalysisPackageSummary, AiAnalysisResponsePreview, AiAnalysisSuggestionRecord,
    AiSuggestionDecisionDraft, AnalyticsOverview, AnalyticsRefreshRequest, BackgroundJob,
    CompetitionKind, DataQualityDecisionDraft, DataQualityFinding, EnqueueJobDraft, MatchContext,
    ModelIdentity, ParameterLifecycleReadiness, ParameterLifecycleReadinessRequest,
    ParameterPromotionDecisionRecord, ParameterPromotionRequest, ParameterReplayFixture,
    ParameterRollbackRequest, ParameterShadowValidationRecord, ParameterShadowValidationRequest,
    ParameterTuningCandidateRecord, ParameterTuningDecisionDraft, ParameterTuningDraft,
};
use football_model_api::ModelRequest;
use football_persistence_postgres::PostgresStore;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use uuid::Uuid;

impl ApplicationService {
    pub async fn analytics_overview(&self) -> ApplicationResult<AnalyticsOverview> {
        Ok(self.active_store().await?.analytics_overview().await?)
    }

    pub async fn enqueue_analysis_job(
        &self,
        draft: EnqueueJobDraft,
    ) -> ApplicationResult<BackgroundJob> {
        let store = self.active_store().await?;
        let job = store.enqueue_job(&draft).await?;
        spawn_job_worker(store);
        Ok(job)
    }

    pub async fn list_background_jobs(&self, limit: u32) -> ApplicationResult<Vec<BackgroundJob>> {
        Ok(self.active_store().await?.list_jobs(limit).await?)
    }

    pub async fn cancel_background_job(&self, job_id: Uuid) -> ApplicationResult<BackgroundJob> {
        Ok(self
            .active_store()
            .await?
            .request_job_cancellation(job_id)
            .await?)
    }

    pub async fn retry_background_job(&self, job_id: Uuid) -> ApplicationResult<BackgroundJob> {
        let store = self.active_store().await?;
        let job = store.retry_job(job_id).await?;
        spawn_job_worker(store);
        Ok(job)
    }

    pub async fn decide_data_quality_finding(
        &self,
        draft: DataQualityDecisionDraft,
    ) -> ApplicationResult<DataQualityFinding> {
        Ok(self
            .active_store()
            .await?
            .decide_data_quality_finding(&draft)
            .await?)
    }

    pub fn export_ai_analysis_response_template(
        &self,
        output_path: String,
        source_package_id: Option<Uuid>,
    ) -> ApplicationResult<String> {
        let bytes = response_template_bytes(source_package_id)?;
        fs::write(&output_path, bytes)?;
        Ok(output_path)
    }

    pub async fn export_ai_analysis_package(
        &self,
        output_path: String,
    ) -> ApplicationResult<AiAnalysisPackageSummary> {
        let store = self.active_store().await?;
        let data = store.build_ai_analysis_data().await?;
        let summary = write_analysis_package(Path::new(&output_path), &data)?;
        store.record_ai_export(&summary).await?;
        Ok(summary)
    }

    pub fn preview_ai_analysis_response(
        &self,
        input_path: String,
    ) -> ApplicationResult<AiAnalysisResponsePreview> {
        Ok(read_analysis_response(Path::new(&input_path))?)
    }

    pub async fn import_ai_analysis_response(
        &self,
        input_path: String,
    ) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>> {
        let preview = read_analysis_response(Path::new(&input_path))?;
        Ok(self
            .active_store()
            .await?
            .import_ai_response(&input_path, &preview)
            .await?)
    }

    pub async fn list_ai_analysis_suggestions(
        &self,
        status: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_ai_suggestions(status.as_deref(), limit)
            .await?)
    }

    pub async fn decide_ai_analysis_suggestion(
        &self,
        draft: AiSuggestionDecisionDraft,
    ) -> ApplicationResult<AiAnalysisSuggestionRecord> {
        Ok(self
            .active_store()
            .await?
            .decide_ai_suggestion(&draft)
            .await?)
    }

    pub async fn generate_parameter_tuning_candidate(
        &self,
        draft: ParameterTuningDraft,
    ) -> ApplicationResult<ParameterTuningCandidateRecord> {
        let _ = draft;
        Err(ApplicationError::Model(
            "公开仓库未捆绑模型提供器，不能生成提供器私有参数候选".to_string(),
        ))
    }

    pub async fn list_parameter_tuning_candidates(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<ParameterTuningCandidateRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_parameter_tuning_candidates(limit)
            .await?)
    }

    pub async fn decide_parameter_tuning_candidate(
        &self,
        draft: ParameterTuningDecisionDraft,
    ) -> ApplicationResult<ParameterTuningCandidateRecord> {
        Ok(self
            .active_store()
            .await?
            .decide_parameter_tuning_candidate(&draft)
            .await?)
    }
}

#[derive(Debug, Clone, Copy)]
struct LifecycleSplit {
    validation_end: usize,
}

#[derive(Debug, Clone)]
struct LifecycleObservation {
    actual_outcome: &'static str,
    home_win: f64,
    draw: f64,
    away_win: f64,
    scoreline_probability: Option<f64>,
}

#[derive(Debug, Clone)]
struct LifecycleMetrics {
    sample_count: u64,
    average_log_loss: f64,
    average_brier: f64,
    average_scoreline_nll: Option<f64>,
    expected_calibration_error: f64,
    home_bias: f64,
    draw_bias: f64,
    away_bias: f64,
}

impl LifecycleMetrics {
    fn as_json(&self) -> Value {
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

fn validate_parameter_horizon(snapshot_type: &str) -> ApplicationResult<()> {
    if matches!(snapshot_type, "T-N" | "T-24h" | "T-6h" | "T-1h") {
        Ok(())
    } else {
        Err(ApplicationError::Validation(format!(
            "P4 参数收敛只允许 T-N、T-24h、T-6h 或 T-1h，收到：{snapshot_type}"
        )))
    }
}

fn lifecycle_split(fixtures: &[ParameterReplayFixture]) -> ApplicationResult<LifecycleSplit> {
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

fn validate_baseline_probabilities(fixtures: &[ParameterReplayFixture]) -> ApplicationResult<()> {
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

fn baseline_metrics(fixtures: &[ParameterReplayFixture]) -> LifecycleMetrics {
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

fn lifecycle_metrics(observations: &[LifecycleObservation]) -> LifecycleMetrics {
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

fn actual_outcome(home_goals: i16, away_goals: i16) -> &'static str {
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

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn competition_kind_from_str(value: &str) -> CompetitionKind {
    match value {
        "league" => CompetitionKind::League,
        "group_stage" => CompetitionKind::GroupStage,
        "knockout_single_leg" => CompetitionKind::KnockoutSingleLeg,
        "knockout_two_leg" => CompetitionKind::KnockoutTwoLeg,
        "friendly" => CompetitionKind::Friendly,
        _ => CompetitionKind::Custom,
    }
}

fn scoreline_probability(payload: &Value, home_goals: i16, away_goals: i16) -> Option<f64> {
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

fn validation_key(
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

fn required_candidate_uuid(value: Option<Uuid>, label: &str) -> ApplicationResult<Uuid> {
    value.ok_or_else(|| ApplicationError::Validation(format!("候选缺少{label}")))
}

fn minimum_sample_size(candidate: &ParameterTuningCandidateRecord) -> u64 {
    candidate
        .constraints
        .get("minimum_sample_size")
        .and_then(Value::as_u64)
        .unwrap_or(50)
}

impl ApplicationService {
    pub async fn parameter_lifecycle_readiness(
        &self,
        request: ParameterLifecycleReadinessRequest,
    ) -> ApplicationResult<ParameterLifecycleReadiness> {
        validate_parameter_horizon(&request.snapshot_type)?;
        if request.minimum_sample_size < 20 {
            return Err(ApplicationError::Validation(
                "最低样本量不能少于 20 场".to_string(),
            ));
        }
        Ok(self
            .active_store()
            .await?
            .parameter_lifecycle_readiness(&request)
            .await?)
    }

    pub async fn run_parameter_shadow_validation(
        &self,
        request: ParameterShadowValidationRequest,
    ) -> ApplicationResult<ParameterShadowValidationRecord> {
        let store = self.active_store().await?;
        let candidate = store
            .read_parameter_tuning_candidate(request.candidate_id)
            .await?;
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
        let readiness = store
            .parameter_lifecycle_readiness(&readiness_request)
            .await?;
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
            return Ok(store.save_parameter_shadow_validation(&record).await?);
        }

        let mut fixtures = store
            .load_parameter_replay_fixtures(
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
        let candidate_parameters = store
            .read_parameter_set_definition(candidate_parameter_set_id)
            .await?;
        let model = self
            .registry
            .get(&candidate.model_key)
            .ok_or_else(|| ApplicationError::ModelNotFound(candidate.model_key.clone()))?;
        model
            .validate_parameters(&candidate_parameters)
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
                    parameters: candidate_parameters.clone(),
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
                actual_outcome: actual_outcome(
                    fixture.actual_home_goals,
                    fixture.actual_away_goals,
                ),
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
        Ok(store.save_parameter_shadow_validation(&record).await?)
    }

    pub async fn list_parameter_shadow_validations(
        &self,
        candidate_id: Uuid,
    ) -> ApplicationResult<Vec<ParameterShadowValidationRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_parameter_shadow_validations(candidate_id)
            .await?)
    }

    pub async fn promote_parameter_candidate(
        &self,
        request: ParameterPromotionRequest,
    ) -> ApplicationResult<ParameterPromotionDecisionRecord> {
        if request.decision_note.trim().len() < 8 {
            return Err(ApplicationError::Validation(
                "人工晋升说明至少需要 8 个字符".to_string(),
            ));
        }
        let store = self.active_store().await?;
        let candidate = store
            .read_parameter_tuning_candidate(request.candidate_id)
            .await?;
        let competition_id = required_candidate_uuid(candidate.competition_id, "赛事范围")?;
        let readiness = store
            .parameter_lifecycle_readiness(&ParameterLifecycleReadinessRequest {
                competition_id: Some(competition_id),
                snapshot_type: candidate.snapshot_type.clone(),
                minimum_sample_size: minimum_sample_size(&candidate),
            })
            .await?;
        if !readiness.ready_for_promotion {
            return Err(ApplicationError::Validation(format!(
                "晋升门禁未通过：{}",
                readiness.blocked_reasons.join("；")
            )));
        }
        if readiness.active_model_version_id != candidate.baseline_model_version_id
            || readiness.active_parameter_set_id != candidate.baseline_parameter_set_id
        {
            return Err(ApplicationError::Validation(
                "正式绑定已发生变化，候选必须重新校准和影子验证".to_string(),
            ));
        }
        Ok(store.promote_parameter_candidate(&request).await?)
    }

    pub async fn rollback_parameter_candidate(
        &self,
        request: ParameterRollbackRequest,
    ) -> ApplicationResult<ParameterPromotionDecisionRecord> {
        if request.decision_note.trim().len() < 8 {
            return Err(ApplicationError::Validation(
                "回滚说明至少需要 8 个字符".to_string(),
            ));
        }
        Ok(self
            .active_store()
            .await?
            .rollback_parameter_candidate(&request)
            .await?)
    }

    pub async fn list_parameter_promotion_decisions(
        &self,
        candidate_id: Uuid,
    ) -> ApplicationResult<Vec<ParameterPromotionDecisionRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_parameter_promotion_decisions(candidate_id)
            .await?)
    }
}

pub(crate) fn spawn_job_worker(store: PostgresStore) {
    tokio::spawn(async move {
        loop {
            let job = match store
                .claim_next_job_by_types(&[
                    "refresh_analytics",
                    "data_quality_scan",
                    "query_performance_scan",
                    "full_analysis_refresh",
                ])
                .await
            {
                Ok(Some(job)) => job,
                Ok(None) => break,
                Err(_) => break,
            };
            let result = execute_job(&store, &job.job_type, &job.payload, job.id).await;
            match result {
                Ok(value) => {
                    let _ = store.complete_job(job.id, value).await;
                }
                Err(error) => {
                    let _ = store.fail_job(job.id, &error.to_string()).await;
                }
            }
        }
    });
}

async fn execute_job(
    store: &PostgresStore,
    job_type: &str,
    payload: &Value,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    match job_type {
        "refresh_analytics" => {
            let request = parse_refresh_request(payload)?;
            if store
                .update_job_progress(job_id, 15.0, "正在读取已复盘推演", json!({}))
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            let overview = store.refresh_analytics(&request).await?;
            Ok(serde_json::to_value(overview)?)
        }
        "data_quality_scan" => {
            if store
                .update_job_progress(job_id, 20.0, "正在检查数据完整性", json!({}))
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            Ok(serde_json::to_value(store.run_data_quality_scan().await?)?)
        }
        "query_performance_scan" => {
            if store
                .update_job_progress(job_id, 20.0, "正在读取 PostgreSQL 统计信息", json!({}))
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            Ok(serde_json::to_value(
                store.capture_query_performance().await?,
            )?)
        }
        "full_analysis_refresh" => {
            let request = parse_refresh_request(payload)?;
            if store
                .update_job_progress(job_id, 10.0, "正在刷新模型评估", json!({}))
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            let overview = store.refresh_analytics(&request).await?;
            if store
                .update_job_progress(
                    job_id,
                    55.0,
                    "正在扫描数据质量",
                    json!({"sample_size":overview.sample_size}),
                )
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            let quality = store.run_data_quality_scan().await?;
            if store
                .update_job_progress(
                    job_id,
                    82.0,
                    "正在分析数据库查询",
                    json!({"open_findings":quality.open_total}),
                )
                .await?
            {
                return Err(ApplicationError::Validation("任务已取消".to_string()));
            }
            let query = store.capture_query_performance().await?;
            Ok(json!({
                "sample_size": overview.sample_size,
                "expected_calibration_error": overview.expected_calibration_error,
                "quality_findings": quality.open_total,
                "database_size_bytes": query.database_size_bytes,
            }))
        }
        other => Err(ApplicationError::Validation(format!(
            "不支持的后台任务：{other}"
        ))),
    }
}

fn parse_refresh_request(payload: &Value) -> ApplicationResult<AnalyticsRefreshRequest> {
    if payload.is_null() || payload.as_object().is_some_and(|item| item.is_empty()) {
        Ok(AnalyticsRefreshRequest::default())
    } else {
        Ok(serde_json::from_value(payload.clone())?)
    }
}
