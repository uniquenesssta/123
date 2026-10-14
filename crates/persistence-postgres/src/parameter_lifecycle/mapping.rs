use super::policy::non_negative_u64;
use crate::{PersistenceError, PersistenceResult};
use football_domain::{
    ParameterPromotionDecisionRecord, ParameterReplayFixture, ParameterShadowValidationRecord,
    ParameterTuningCandidateRecord,
};
use sqlx::Row;

pub(super) fn parameter_tuning_candidate_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ParameterTuningCandidateRecord> {
    let sample_size = row.try_get::<i64, _>("sample_size")?;
    let sample_size = u64::try_from(sample_size)
        .map_err(|_| PersistenceError::InvalidState("参数候选样本量不能为负数".to_string()))?;
    Ok(ParameterTuningCandidateRecord {
        id: row.try_get("id")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        partition_key: row.try_get("partition_key")?,
        model_key: row.try_get("model_key")?,
        model_version: row.try_get("model_version")?,
        parameter_version: row.try_get("parameter_version")?,
        snapshot_type: row.try_get("snapshot_type")?,
        target_module: row.try_get("target_module")?,
        sample_size,
        baseline_model_version_id: row.try_get("baseline_model_version_id")?,
        baseline_parameter_set_id: row.try_get("baseline_parameter_set_id")?,
        candidate_model_version_id: row.try_get("candidate_model_version_id")?,
        candidate_parameter_set_id: row.try_get("candidate_parameter_set_id")?,
        candidate_model_version: row.try_get("candidate_model_version")?,
        candidate_parameter_version: row.try_get("candidate_parameter_version")?,
        candidate_definition_sha256: row.try_get("candidate_definition_sha256")?,
        baseline_metrics: row.try_get("baseline_metrics")?,
        calibration_bias: row.try_get("calibration_bias")?,
        proposed_adjustments: row.try_get("proposed_adjustments")?,
        constraints: row.try_get("constraints")?,
        training_window: row.try_get("training_window")?,
        validation_window: row.try_get("validation_window")?,
        holdout_window: row.try_get("holdout_window")?,
        rationale: row.try_get("rationale")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
        decided_at: row.try_get("decided_at")?,
        decision_note: row.try_get("decision_note")?,
    })
}

pub(super) fn replay_fixture_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ParameterReplayFixture> {
    Ok(ParameterReplayFixture {
        review_id: row.try_get("review_id")?,
        run_id: row.try_get("run_id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        competition_id: row.try_get("competition_id")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        competition_kind: row.try_get("competition_kind")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        kickoff_time: row.try_get("kickoff_time")?,
        home_team_name: row
            .try_get::<Option<String>, _>("home_team_name")?
            .unwrap_or_else(|| "主队".to_string()),
        away_team_name: row
            .try_get::<Option<String>, _>("away_team_name")?
            .unwrap_or_else(|| "客队".to_string()),
        snapshot_type: row.try_get("snapshot_type")?,
        input_payload: row.try_get("input_payload")?,
        rule_package_version: row.try_get("rule_package_version")?,
        actual_home_goals: row.try_get("actual_home_goals")?,
        actual_away_goals: row.try_get("actual_away_goals")?,
        baseline_home_win: row.try_get("baseline_home_win")?,
        baseline_draw: row.try_get("baseline_draw")?,
        baseline_away_win: row.try_get("baseline_away_win")?,
        baseline_scoreline_probability: row.try_get("baseline_scoreline_probability")?,
        data_coverage: row.try_get("data_coverage")?,
    })
}

pub(super) fn shadow_validation_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ParameterShadowValidationRecord> {
    Ok(ParameterShadowValidationRecord {
        id: row.try_get("id")?,
        candidate_id: row.try_get("candidate_id")?,
        validation_key: row.try_get("validation_key")?,
        partition_key: row.try_get("partition_key")?,
        sample_count: non_negative_u64(row.try_get("sample_count")?, "影子验证样本数")?,
        baseline_metrics: row.try_get("baseline_metrics")?,
        candidate_metrics: row.try_get("candidate_metrics")?,
        metric_deltas: row.try_get("metric_deltas")?,
        gate_results: row.try_get("gate_results")?,
        status: row.try_get("status")?,
        generated_at: row.try_get("generated_at")?,
    })
}

pub(super) fn promotion_decision_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ParameterPromotionDecisionRecord> {
    Ok(ParameterPromotionDecisionRecord {
        id: row.try_get("id")?,
        candidate_id: row.try_get("candidate_id")?,
        decision: row.try_get("decision")?,
        previous_binding_state: row.try_get("previous_binding_state")?,
        new_binding_state: row.try_get("new_binding_state")?,
        decided_by: row.try_get("decided_by")?,
        decision_note: row.try_get("decision_note")?,
        created_at: row.try_get("created_at")?,
    })
}
