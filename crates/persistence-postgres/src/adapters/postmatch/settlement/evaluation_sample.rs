use super::{
    context::SettlementContext,
    validation::{json_f64, json_text, required, validate_probabilities},
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::POSTMATCH_MONITORING_VERSION;
use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct EvaluationSamplePartition<'a> {
    pub(super) competition_profile_id: Uuid,
    pub(super) model_version_id: Uuid,
    pub(super) parameter_set_id: Uuid,
    pub(super) horizon: &'a str,
}

impl PostgresStore {
    pub(super) async fn insert_postmatch_evaluation_sample_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        settlement_id: Uuid,
        context: &SettlementContext,
        partition: EvaluationSamplePartition<'_>,
    ) -> PersistenceResult<()> {
        let EvaluationSamplePartition {
            competition_profile_id,
            model_version_id,
            parameter_set_id,
            horizon,
        } = partition;
        let evaluation = &context.prediction_evaluation;
        if !evaluation
            .get("available")
            .and_then(Value::as_bool)
            .unwrap_or(false)
        {
            return Err(PersistenceError::InvalidState(
                "最终复盘没有可用预测评价，不能进入正式结算样本".to_string(),
            ));
        }
        let run_id = required(context.model_run_id, "复盘未绑定成功推演")?;
        let actual_outcome = json_text(evaluation, "actual_outcome")?;
        let expected_actual_outcome = match (
            required(context.home_goals_90, "正式赛果缺失")?,
            required(context.away_goals_90, "正式赛果缺失")?,
        ) {
            (home, away) if home > away => "home_win",
            (home, away) if home < away => "away_win",
            _ => "draw",
        };
        if actual_outcome != expected_actual_outcome {
            return Err(PersistenceError::InvalidState(
                "最终复盘的实际赛果标签与冻结比分不一致".to_string(),
            ));
        }
        let probabilities = evaluation
            .get("predicted_probabilities")
            .ok_or_else(|| PersistenceError::InvalidState("预测概率快照缺失".to_string()))?;
        let home_win = json_f64(probabilities, "home_win")?;
        let draw = json_f64(probabilities, "draw")?;
        let away_win = json_f64(probabilities, "away_win")?;
        validate_probabilities(home_win, draw, away_win)?;
        let log_loss = json_f64(evaluation, "log_loss")?;
        let brier = json_f64(evaluation, "brier")?;
        let scoreline_nll = evaluation.get("scoreline_nll").and_then(Value::as_f64);
        if log_loss < 0.0
            || !(0.0..=2.0).contains(&brier)
            || scoreline_nll.is_some_and(|value| !value.is_finite() || value < 0.0)
            || !context.data_coverage.is_finite()
            || !(0.0..=1.0).contains(&context.data_coverage)
        {
            return Err(PersistenceError::InvalidState(
                "最终复盘评价指标或数据覆盖率超出合法范围".to_string(),
            ));
        }
        sqlx::query(
            r#"
            INSERT INTO analytics.evaluation_samples (
                id, review_id, run_id, model_version_id, parameter_set_id,
                competition_id, season_id, stage_id, snapshot_type, kickoff_time,
                actual_outcome, home_win, draw, away_win, log_loss, brier,
                scoreline_nll, data_coverage, calculation_version, settlement_id,
                competition_profile_id, calculated_at
            )
            SELECT $1,$2,$3,$4,$5,fixture.competition_id,fixture.season_id,fixture.stage_id,
                   $6,fixture.kickoff_time,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,now()
            FROM football.matches fixture WHERE fixture.id=$18
            ON CONFLICT (run_id, kickoff_time, calculation_version) DO NOTHING
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(context.match_review_id)
        .bind(run_id)
        .bind(model_version_id)
        .bind(parameter_set_id)
        .bind(horizon)
        .bind(actual_outcome)
        .bind(home_win)
        .bind(draw)
        .bind(away_win)
        .bind(log_loss)
        .bind(brier)
        .bind(scoreline_nll)
        .bind(context.data_coverage)
        .bind(POSTMATCH_MONITORING_VERSION)
        .bind(settlement_id)
        .bind(competition_profile_id)
        .bind(context.match_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}
