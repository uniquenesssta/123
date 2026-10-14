use super::mapping::replay_fixture_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::ParameterReplayFixture;
use uuid::Uuid;

impl PostgresStore {
    pub async fn load_parameter_replay_fixtures(
        &self,
        competition_id: Uuid,
        competition_profile_id: Uuid,
        snapshot_type: &str,
        baseline_model_version_id: Uuid,
        baseline_parameter_set_id: Uuid,
    ) -> PersistenceResult<Vec<ParameterReplayFixture>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (run.id)
                review.id AS review_id,
                run.id AS run_id,
                fixture.id AS match_id,
                fixture.external_key AS match_key,
                fixture.competition_id,
                fixture.season_id,
                fixture.stage_id,
                COALESCE(stage.stage_kind, competition.competition_kind, 'custom') AS competition_kind,
                package.competition_profile_id,
                fixture.kickoff_time,
                home.canonical_name AS home_team_name,
                away.canonical_name AS away_team_name,
                run.snapshot_type,
                run.input_payload,
                package.version AS rule_package_version,
                result.home_goals_90 AS actual_home_goals,
                result.away_goals_90 AS actual_away_goals,
                (run.summary->>'home_win')::double precision AS baseline_home_win,
                (run.summary->>'draw')::double precision AS baseline_draw,
                (run.summary->>'away_win')::double precision AS baseline_away_win,
                scoreline.probability AS baseline_scoreline_probability,
                review.data_coverage::double precision AS data_coverage
            FROM review.postmatch_settlements settlement
            JOIN review.match_reviews review ON review.id = settlement.match_review_id
            JOIN football.match_results result ON result.match_id = settlement.match_id
            JOIN football.matches fixture ON fixture.id = settlement.match_id
            JOIN football.competitions competition ON competition.id = fixture.competition_id
            LEFT JOIN football.competition_stages stage ON stage.id = fixture.stage_id
            LEFT JOIN football.teams home ON home.id = fixture.home_team_id
            LEFT JOIN football.teams away ON away.id = fixture.away_team_id
            JOIN model.runs run ON run.id = settlement.model_run_id AND run.status = 'succeeded'
            JOIN model.rule_packages package ON package.id = settlement.rule_package_id
            LEFT JOIN model.run_scorelines scoreline
              ON scoreline.run_id = run.id
             AND scoreline.home_goals = result.home_goals_90
             AND scoreline.away_goals = result.away_goals_90
            WHERE review.status = 'finalized'
              AND COALESCE((review.prediction_evaluation->>'available')::boolean, false) = true
              AND settlement.competition_id = $1
              AND settlement.competition_profile_id = $2
              AND settlement.horizon = $3
              AND settlement.model_version_id = $4
              AND settlement.parameter_set_id = $5
              AND package.competition_profile_id = settlement.competition_profile_id
              AND run.snapshot_type = settlement.horizon
              AND run.model_version_id = settlement.model_version_id
              AND run.parameter_set_id = settlement.parameter_set_id
            ORDER BY run.id, review.created_at DESC, review.id DESC
            "#,
        )
        .bind(competition_id)
        .bind(competition_profile_id)
        .bind(snapshot_type)
        .bind(baseline_model_version_id)
        .bind(baseline_parameter_set_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(replay_fixture_from_row).collect()
    }
}
