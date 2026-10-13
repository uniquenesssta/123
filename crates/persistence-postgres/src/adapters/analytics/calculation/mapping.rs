use crate::PersistenceResult;
use football_domain::EvaluationSample;
use sqlx::Row;

pub(super) fn evaluation_sample_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EvaluationSample> {
    Ok(EvaluationSample {
        review_id: row.try_get("review_id")?,
        run_id: row.try_get("run_id")?,
        model_version_id: row.try_get("model_version_id")?,
        parameter_set_id: row.try_get("parameter_set_id")?,
        model_key: row.try_get("model_key")?,
        model_version: row.try_get("model_version")?,
        parameter_version: row.try_get("parameter_version")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        snapshot_type: row.try_get("snapshot_type")?,
        kickoff_time: row.try_get("kickoff_time")?,
        actual_outcome: row.try_get("actual_outcome")?,
        home_win: row.try_get("home_win")?,
        draw: row.try_get("draw")?,
        away_win: row.try_get("away_win")?,
        log_loss: row.try_get("log_loss")?,
        brier: row.try_get("brier")?,
        scoreline_nll: row.try_get("scoreline_nll")?,
        data_coverage: row.try_get("data_coverage")?,
    })
}
