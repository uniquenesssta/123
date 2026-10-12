use crate::{PersistenceError, PersistenceResult};
use chrono::{DateTime, Utc};
use football_domain::ReviewPlayerBaseline;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

pub(super) async fn load_review_players_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    match_id: Uuid,
) -> PersistenceResult<Vec<ReviewPlayerBaseline>> {
    let rows = sqlx::query(
        r#"
        SELECT
            observation.id AS observation_id,
            observation.player_id, player.canonical_name AS player_name,
            observation.team_id, team.canonical_name AS team_name,
            observation.position_code, observation.role_code,
            observation.started, observation.minutes_played,
            observation.performance_score, observation.input_confidence,
            observation.metrics,
            COALESCE(contribution.effective_contribution, profile.average_value, 50)::double precision AS expected_performance,
            COALESCE(contribution.confidence, profile.average_confidence, 0.50)::double precision AS expected_confidence,
            COALESCE(profile.abilities, '{}'::jsonb) AS current_abilities,
            COALESCE(history.reviewed_match_count, 0)::integer AS reviewed_match_count,
            COALESCE(history.substitute_appearances, 0)::integer AS substitute_appearances
        FROM review.player_match_observations observation
        JOIN football.players player ON player.id = observation.player_id
        JOIN football.teams team ON team.id = observation.team_id
        LEFT JOIN LATERAL (
            SELECT snapshot.effective_contribution, snapshot.confidence
            FROM feature.match_player_contributions snapshot
            WHERE snapshot.match_id = observation.match_id
              AND snapshot.player_id = observation.player_id
            ORDER BY snapshot.as_of DESC, snapshot.created_at DESC
            LIMIT 1
        ) contribution ON true
        LEFT JOIN feature.player_ability_profiles profile ON profile.player_id = observation.player_id
        LEFT JOIN LATERAL (
            SELECT
                COUNT(DISTINCT match_review.match_id)::integer AS reviewed_match_count,
                COUNT(DISTINCT match_review.match_id) FILTER (
                    WHERE player_review.entry_type = 'substitute'
                )::integer AS substitute_appearances
            FROM review.player_match_reviews player_review
            JOIN review.match_reviews match_review
              ON match_review.id = player_review.match_review_id
            WHERE player_review.player_id = observation.player_id
              AND match_review.match_id <> observation.match_id
              AND match_review.status = 'finalized'
        ) history ON true
        WHERE observation.match_id = $1
        ORDER BY observation.team_id, observation.started DESC, observation.minutes_played DESC
        "#,
    )
    .bind(match_id)
    .fetch_all(&mut **tx)
    .await?;
    rows.iter()
        .map(|row| {
            Ok(ReviewPlayerBaseline {
                observation_id: row.try_get("observation_id")?,
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                position_code: row.try_get("position_code")?,
                role_code: row.try_get("role_code")?,
                started: row.try_get("started")?,
                minutes_played: row.try_get("minutes_played")?,
                performance_score: row.try_get("performance_score")?,
                input_confidence: row.try_get("input_confidence")?,
                metrics: serde_json::from_value(row.try_get("metrics")?)?,
                expected_performance: row.try_get("expected_performance")?,
                expected_confidence: row.try_get("expected_confidence")?,
                current_abilities: row.try_get("current_abilities")?,
                reviewed_match_count: row.try_get("reviewed_match_count")?,
                substitute_appearances: row.try_get("substitute_appearances")?,
            })
        })
        .collect()
}

pub(super) async fn calculate_recent_starter_overlap(
    tx: &mut Transaction<'_, Postgres>,
    match_id: Uuid,
    team_id: Uuid,
    kickoff_time: DateTime<Utc>,
) -> PersistenceResult<f64> {
    let value: f64 = sqlx::query_scalar(
        r#"
        WITH current_starters AS (
            SELECT player_id
            FROM review.player_match_observations
            WHERE match_id = $1 AND team_id = $2 AND started
        ), previous_lineups AS (
            SELECT DISTINCT ON (lineup.match_id)
                lineup.id, lineup.match_id, fixture.kickoff_time
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id = lineup.match_id
            WHERE lineup.team_id = $2
              AND lineup.lineup_type = 'actual'
              AND lineup.status = 'active'
              AND fixture.id <> $1
              AND fixture.kickoff_time < $3
            ORDER BY lineup.match_id, lineup.captured_at DESC, lineup.id DESC
        ), recent_lineups AS (
            SELECT * FROM previous_lineups
            ORDER BY kickoff_time DESC
            LIMIT 10
        ), overlap_by_lineup AS (
            SELECT
                recent.id,
                COUNT(lineup_player.player_id) FILTER (
                    WHERE lineup_player.player_id IN (SELECT player_id FROM current_starters)
                      AND lineup_player.is_starter
                )::double precision
                / GREATEST((SELECT COUNT(*) FROM current_starters), 1)::double precision AS overlap
            FROM recent_lineups recent
            JOIN football.lineup_players lineup_player ON lineup_player.lineup_id = recent.id
            GROUP BY recent.id
        )
        SELECT COALESCE(AVG(overlap), 0.50)::double precision
        FROM overlap_by_lineup
        "#,
    )
    .bind(match_id)
    .bind(team_id)
    .bind(kickoff_time)
    .fetch_one(&mut **tx)
    .await?;
    Ok(value.clamp(0.0, 1.0))
}

pub(super) async fn load_prediction_context_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    match_id: Uuid,
    source_run_id: Option<Uuid>,
    home_goals: i16,
    away_goals: i16,
) -> PersistenceResult<Option<football_domain::PredictionReviewContext>> {
    let row = if let Some(run_id) = source_run_id {
        sqlx::query(
            "SELECT id, summary FROM model.runs WHERE id=$1 AND match_id=$2 AND status='succeeded'",
        )
        .bind(run_id)
        .bind(match_id)
        .fetch_optional(&mut **tx)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT id, summary
            FROM model.runs
            WHERE match_id=$1 AND status='succeeded'
            ORDER BY completed_at DESC NULLS LAST, created_at DESC
            LIMIT 1
            "#,
        )
        .bind(match_id)
        .fetch_optional(&mut **tx)
        .await?
    };
    let Some(row) = row else {
        if source_run_id.is_some() {
            return Err(PersistenceError::InvalidState(
                "指定的模型运行不存在、未成功或不属于该场比赛".to_string(),
            ));
        }
        return Ok(None);
    };
    let run_id: Uuid = row.try_get("id")?;
    let scoreline_probability: Option<f64> = sqlx::query_scalar(
        "SELECT probability FROM model.run_scorelines WHERE run_id=$1 AND home_goals=$2 AND away_goals=$3",
    )
    .bind(run_id)
    .bind(home_goals)
    .bind(away_goals)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(Some(football_domain::PredictionReviewContext {
        run_id,
        summary: row.try_get("summary")?,
        actual_scoreline_probability: scoreline_probability,
    }))
}
