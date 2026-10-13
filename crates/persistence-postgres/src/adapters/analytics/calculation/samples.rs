use super::mapping::evaluation_sample_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{AnalyticsRefreshRequest, EvaluationSample};

impl PostgresStore {
    pub(super) async fn load_evaluation_samples(
        &self,
        request: &AnalyticsRefreshRequest,
    ) -> PersistenceResult<Vec<EvaluationSample>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (run.id)
                review.id AS review_id, run.id AS run_id,
                run.model_version_id, run.parameter_set_id,
                definition.model_key, version.version AS model_version,
                parameter.parameter_version,
                fixture.competition_id, competition.name AS competition_name,
                fixture.season_id, fixture.stage_id, run.snapshot_type, fixture.kickoff_time,
                review.prediction_evaluation->>'actual_outcome' AS actual_outcome,
                (review.prediction_evaluation->'predicted_probabilities'->>'home_win')::double precision AS home_win,
                (review.prediction_evaluation->'predicted_probabilities'->>'draw')::double precision AS draw,
                (review.prediction_evaluation->'predicted_probabilities'->>'away_win')::double precision AS away_win,
                (review.prediction_evaluation->>'log_loss')::double precision AS log_loss,
                (review.prediction_evaluation->>'brier')::double precision AS brier,
                NULLIF(review.prediction_evaluation->>'scoreline_nll','')::double precision AS scoreline_nll,
                review.data_coverage::double precision AS data_coverage
            FROM review.match_reviews review
            JOIN model.runs run ON run.id=review.source_run_id AND run.status='succeeded'
            JOIN model.versions version ON version.id=run.model_version_id
            JOIN model.definitions definition ON definition.id=version.model_id
            JOIN model.parameter_sets parameter ON parameter.id=run.parameter_set_id
            JOIN football.matches fixture ON fixture.id=review.match_id
            LEFT JOIN football.competitions competition ON competition.id=fixture.competition_id
            WHERE review.status='finalized'
              AND COALESCE((review.prediction_evaluation->>'available')::boolean,false)=true
              AND ($1::uuid IS NULL OR fixture.competition_id=$1)
              AND ($2::timestamptz IS NULL OR fixture.kickoff_time >= $2)
              AND ($3::timestamptz IS NULL OR fixture.kickoff_time <= $3)
            ORDER BY run.id, review.created_at DESC, review.id DESC
            "#,
        )
        .bind(request.competition_id)
        .bind(request.window_start)
        .bind(request.window_end)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(evaluation_sample_from_row).collect()
    }
}
