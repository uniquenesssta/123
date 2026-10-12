use super::{PersistenceError, PersistenceResult, PostgresStore};
use crate::adapters::review::facts::summarize_match_events;
use football_domain::{
    MatchRecord, MatchResultRecord, MatchReviewDetail, MatchReviewSummary, PlayerMatchReviewRecord,
    ReviewableMatch, SubstitutionRecord, TeamMatchReviewRecord,
};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_reviewable_matches(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<ReviewableMatch>> {
        let rows = sqlx::query(
            r#"
            WITH selected_fixtures AS (
                SELECT fixture.*
                FROM football.matches fixture
                WHERE fixture.status IN ('live', 'finished')
                   OR fixture.kickoff_time <= now()
                   OR EXISTS (
                       SELECT 1 FROM football.match_results result
                       WHERE result.match_id = fixture.id
                   )
                ORDER BY fixture.kickoff_time DESC, fixture.id DESC
                LIMIT $1
            )
            SELECT
                fixture.id, fixture.external_key,
                fixture.competition_id, competition.name AS competition_name,
                fixture.season_id, fixture.stage_id, fixture.round_id,
                fixture.home_team_id, home.canonical_name AS home_team_name,
                fixture.away_team_id, away.canonical_name AS away_team_name,
                fixture.kickoff_time, fixture.status, fixture.venue,
                result.home_goals_90, result.away_goals_90,
                result.home_goals_extra_time, result.away_goals_extra_time,
                result.home_penalties, result.away_penalties,
                result.finalized_at, result.metadata AS result_metadata,
                latest_review.id AS review_id,
                latest_review.review_version,
                latest_review.status AS review_status,
                latest_review.data_coverage::double precision AS review_data_coverage,
                latest_review.source_run_id,
                latest_review.calculation_version,
                latest_review.result_snapshot,
                latest_review.substitutions_snapshot,
                latest_review.prediction_evaluation,
                latest_review.conclusions,
                latest_review.created_at AS review_created_at,
                latest_review.finalized_at AS review_finalized_at,
                COALESCE(observation_count.value, 0)::bigint AS player_observation_count,
                COALESCE(lineup_count.value, 0)::bigint AS actual_lineup_count
            FROM selected_fixtures fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            LEFT JOIN football.match_results result ON result.match_id = fixture.id
            LEFT JOIN LATERAL (
                SELECT review.*
                FROM review.match_reviews review
                WHERE review.match_id = fixture.id
                ORDER BY review.created_at DESC, review.id DESC
                LIMIT 1
            ) latest_review ON true
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS value
                FROM review.player_match_observations observation
                WHERE observation.match_id = fixture.id
            ) observation_count ON true
            LEFT JOIN LATERAL (
                SELECT COUNT(*) AS value
                FROM football.lineups lineup
                WHERE lineup.match_id = fixture.id
                  AND lineup.lineup_type = 'actual'
                  AND lineup.status = 'active'
            ) lineup_count ON true
            ORDER BY fixture.kickoff_time DESC, fixture.id DESC
            "#,
        )
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(reviewable_match_from_row).collect()
    }

    pub async fn list_match_reviews(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<MatchReviewSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT
                review.id, review.match_id, fixture.external_key AS match_key,
                home.canonical_name AS home_team_name,
                away.canonical_name AS away_team_name,
                review.review_version, review.status,
                review.data_coverage::double precision AS data_coverage,
                review.source_run_id, review.calculation_version,
                review.result_snapshot, review.substitutions_snapshot, review.prediction_evaluation,
                review.conclusions, review.created_at, review.finalized_at
            FROM review.match_reviews review
            JOIN football.matches fixture ON fixture.id = review.match_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            ORDER BY review.created_at DESC, review.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(match_review_summary_from_row).collect()
    }

    pub async fn read_match_review(&self, review_id: Uuid) -> PersistenceResult<MatchReviewDetail> {
        let summary_row = sqlx::query(
            r#"
            SELECT
                review.id, review.match_id, fixture.external_key AS match_key,
                home.canonical_name AS home_team_name,
                away.canonical_name AS away_team_name,
                review.review_version, review.status,
                review.data_coverage::double precision AS data_coverage,
                review.source_run_id, review.calculation_version,
                review.result_snapshot, review.substitutions_snapshot, review.prediction_evaluation,
                review.conclusions, review.created_at, review.finalized_at
            FROM review.match_reviews review
            JOIN football.matches fixture ON fixture.id = review.match_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            WHERE review.id = $1
            "#,
        )
        .bind(review_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("复盘记录不存在".to_string()))?;
        let summary = match_review_summary_from_row(&summary_row)?;
        let result: MatchResultRecord = serde_json::from_value(summary.result_snapshot.clone())?;
        let substitutions: Vec<SubstitutionRecord> =
            serde_json::from_value(summary.substitutions_snapshot.clone())?;

        let player_rows = sqlx::query(
            r#"
            SELECT
                player_review.id, player_review.match_review_id,
                player_review.player_id, player.canonical_name AS player_name,
                player_review.team_id, team.canonical_name AS team_name,
                player_review.role_code, player_review.started, player_review.entry_type,
                player_review.minutes_played, player_review.expected_performance,
                player_review.actual_performance, player_review.realization_ratio,
                player_review.confidence, player_review.contribution_weight,
                player_review.ability_candidate_count, player_review.metrics
            FROM review.player_match_reviews player_review
            JOIN football.players player ON player.id = player_review.player_id
            JOIN football.teams team ON team.id = player_review.team_id
            WHERE player_review.match_review_id = $1
            ORDER BY player_review.team_id, player_review.started DESC,
                     player_review.minutes_played DESC NULLS LAST, player.canonical_name
            "#,
        )
        .bind(review_id)
        .fetch_all(&self.pool)
        .await?;
        let player_reviews = player_rows
            .iter()
            .map(player_match_review_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;

        let team_rows = sqlx::query(
            r#"
            SELECT
                team_review.id, team_review.match_review_id,
                team_review.team_id, team.canonical_name AS team_name,
                team_review.chemistry_score, team_review.lineup_continuity,
                team_review.performance_cohesion, team_review.bench_strength,
                team_review.bench_dropoff, team_review.substitution_impact,
                team_review.substitute_count, team_review.realization_score,
                team_review.confidence, team_review.metrics
            FROM review.team_match_reviews team_review
            JOIN football.teams team ON team.id = team_review.team_id
            WHERE team_review.match_review_id = $1
            ORDER BY team.canonical_name
            "#,
        )
        .bind(review_id)
        .fetch_all(&self.pool)
        .await?;
        let team_reviews = team_rows
            .iter()
            .map(team_match_review_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        let events = self.list_match_events(summary.match_id).await?;
        let event_summary = summarize_match_events(&events);
        let ability_candidates = self
            .list_ability_candidates(None, 1000, Some(review_id))
            .await?;

        Ok(MatchReviewDetail {
            summary,
            result,
            substitutions,
            events,
            event_summary,
            player_reviews,
            team_reviews,
            ability_candidates,
        })
    }
}

fn reviewable_match_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<ReviewableMatch> {
    let match_record = match_record_from_review_row(row)?;
    let result = if row.try_get::<Option<i16>, _>("home_goals_90")?.is_some() {
        Some(MatchResultRecord {
            match_id: match_record.id,
            home_goals_90: row.try_get("home_goals_90")?,
            away_goals_90: row.try_get("away_goals_90")?,
            home_goals_extra_time: row.try_get("home_goals_extra_time")?,
            away_goals_extra_time: row.try_get("away_goals_extra_time")?,
            home_penalties: row.try_get("home_penalties")?,
            away_penalties: row.try_get("away_penalties")?,
            finalized_at: row.try_get("finalized_at")?,
            metadata: row.try_get("result_metadata")?,
        })
    } else {
        None
    };
    let latest_review = if row.try_get::<Option<Uuid>, _>("review_id")?.is_some() {
        Some(MatchReviewSummary {
            id: row.try_get("review_id")?,
            match_id: match_record.id,
            match_key: match_record.external_key.clone(),
            home_team_name: match_record.home_team_name.clone(),
            away_team_name: match_record.away_team_name.clone(),
            review_version: row.try_get("review_version")?,
            status: row.try_get("review_status")?,
            data_coverage: row.try_get("review_data_coverage")?,
            source_run_id: row.try_get("source_run_id")?,
            calculation_version: row.try_get("calculation_version")?,
            result_snapshot: row.try_get("result_snapshot")?,
            substitutions_snapshot: row.try_get("substitutions_snapshot")?,
            prediction_evaluation: row.try_get("prediction_evaluation")?,
            conclusions: row.try_get("conclusions")?,
            created_at: row.try_get("review_created_at")?,
            finalized_at: row.try_get("review_finalized_at")?,
        })
    } else {
        None
    };
    Ok(ReviewableMatch {
        match_record,
        result,
        latest_review,
        player_observation_count: row.try_get("player_observation_count")?,
        actual_lineup_count: row.try_get("actual_lineup_count")?,
    })
}

pub(crate) fn match_record_from_review_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<MatchRecord> {
    Ok(MatchRecord {
        id: row.try_get("id")?,
        external_key: row.try_get("external_key")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        round_id: row.try_get("round_id")?,
        home_team_id: row.try_get("home_team_id")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_id: row.try_get("away_team_id")?,
        away_team_name: row.try_get("away_team_name")?,
        kickoff_time: row.try_get("kickoff_time")?,
        status: match row.try_get::<String, _>("status")?.as_str() {
            "scheduled" => football_domain::MatchStatus::Scheduled,
            "live" => football_domain::MatchStatus::Live,
            "finished" => football_domain::MatchStatus::Finished,
            "postponed" => football_domain::MatchStatus::Postponed,
            "cancelled" => football_domain::MatchStatus::Cancelled,
            other => {
                return Err(PersistenceError::InvalidState(format!(
                    "未知比赛状态：{other}"
                )))
            }
        },
        venue: row.try_get("venue")?,
    })
}

fn match_review_summary_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<MatchReviewSummary> {
    Ok(MatchReviewSummary {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_name: row.try_get("away_team_name")?,
        review_version: row.try_get("review_version")?,
        status: row.try_get("status")?,
        data_coverage: row.try_get("data_coverage")?,
        source_run_id: row.try_get("source_run_id")?,
        calculation_version: row.try_get("calculation_version")?,
        result_snapshot: row.try_get("result_snapshot")?,
        substitutions_snapshot: row.try_get("substitutions_snapshot")?,
        prediction_evaluation: row.try_get("prediction_evaluation")?,
        conclusions: row.try_get("conclusions")?,
        created_at: row.try_get("created_at")?,
        finalized_at: row.try_get("finalized_at")?,
    })
}

fn player_match_review_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerMatchReviewRecord> {
    Ok(PlayerMatchReviewRecord {
        id: row.try_get("id")?,
        match_review_id: row.try_get("match_review_id")?,
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        role_code: row.try_get("role_code")?,
        started: row.try_get("started")?,
        entry_type: row.try_get("entry_type")?,
        minutes_played: row.try_get("minutes_played")?,
        expected_performance: row.try_get("expected_performance")?,
        actual_performance: row.try_get("actual_performance")?,
        realization_ratio: row.try_get("realization_ratio")?,
        confidence: row.try_get("confidence")?,
        contribution_weight: row.try_get("contribution_weight")?,
        ability_candidate_count: row.try_get("ability_candidate_count")?,
        metrics: row.try_get("metrics")?,
    })
}

fn team_match_review_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<TeamMatchReviewRecord> {
    Ok(TeamMatchReviewRecord {
        id: row.try_get("id")?,
        match_review_id: row.try_get("match_review_id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        chemistry_score: row.try_get("chemistry_score")?,
        lineup_continuity: row.try_get("lineup_continuity")?,
        performance_cohesion: row.try_get("performance_cohesion")?,
        bench_strength: row.try_get("bench_strength")?,
        bench_dropoff: row.try_get("bench_dropoff")?,
        substitution_impact: row.try_get("substitution_impact")?,
        substitute_count: row.try_get("substitute_count")?,
        realization_score: row.try_get("realization_score")?,
        confidence: row.try_get("confidence")?,
        metrics: row.try_get("metrics")?,
    })
}
