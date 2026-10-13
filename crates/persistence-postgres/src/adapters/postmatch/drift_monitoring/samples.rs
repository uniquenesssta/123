use super::super::monitoring::PostmatchPartition;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{EvaluationSample, POSTMATCH_MONITORING_VERSION};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub(super) async fn load_drift_samples(
        &self,
        partition: &PostmatchPartition,
    ) -> PersistenceResult<(Vec<EvaluationSample>, Vec<Uuid>)> {
        let rows = sqlx::query(
            r#"
            SELECT sample.review_id, sample.run_id, sample.model_version_id,
                   sample.parameter_set_id, definition.model_key,
                   version.version AS model_version, parameter.parameter_version,
                   sample.competition_id, competition.name AS competition_name,
                   sample.season_id, sample.stage_id, sample.snapshot_type,
                   sample.kickoff_time, sample.actual_outcome, sample.home_win,
                   sample.draw, sample.away_win, sample.log_loss, sample.brier,
                   sample.scoreline_nll, sample.data_coverage, settlement.id AS settlement_id
            FROM analytics.evaluation_samples sample
            JOIN review.postmatch_settlements settlement ON settlement.id=sample.settlement_id
            JOIN model.versions version ON version.id=sample.model_version_id
            JOIN model.definitions definition ON definition.id=version.model_id
            JOIN model.parameter_sets parameter ON parameter.id=sample.parameter_set_id
            LEFT JOIN football.competitions competition ON competition.id=sample.competition_id
            WHERE sample.competition_id=$1
              AND sample.competition_profile_id=$2
              AND sample.model_version_id=$3
              AND sample.parameter_set_id=$4
              AND sample.snapshot_type=$5
              AND sample.calculation_version=$6
            ORDER BY sample.kickoff_time, sample.run_id
            "#,
        )
        .bind(partition.competition_id)
        .bind(partition.competition_profile_id)
        .bind(partition.model_version_id)
        .bind(partition.parameter_set_id)
        .bind(&partition.horizon)
        .bind(POSTMATCH_MONITORING_VERSION)
        .fetch_all(&self.pool)
        .await?;
        let mut samples = Vec::with_capacity(rows.len());
        let mut settlement_ids = Vec::with_capacity(rows.len());
        for row in rows {
            settlement_ids.push(row.try_get::<Uuid, _>("settlement_id")?);
            samples.push(EvaluationSample {
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
            });
        }
        Ok((samples, settlement_ids))
    }
}
