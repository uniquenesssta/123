use crate::team_features::{
    finite_or_default, GoalBaseline, HistoricalMatch, DEFAULT_GOAL_BASELINE,
};
use crate::PersistenceResult;
use chrono::{DateTime, Utc};
use football_domain::MatchRecord;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

const HISTORY_QUERY_LIMIT: i64 = 36;

#[derive(FromRow)]
struct HistoricalResultRow {
    kickoff_time: DateTime<Utc>,
    home_team_id: Uuid,
    competition_id: Option<Uuid>,
    season_id: Option<Uuid>,
    home_goals_90: i16,
    away_goals_90: i16,
}

#[derive(FromRow)]
struct GoalBaselineRow {
    home_goals: Option<f64>,
    away_goals: Option<f64>,
    match_count: i64,
}

pub(super) async fn team_history(
    pool: &PgPool,
    fixture: &MatchRecord,
    team_id: Uuid,
    is_home: bool,
    data_cutoff_time: DateTime<Utc>,
) -> PersistenceResult<Vec<HistoricalMatch>> {
    let rows = sqlx::query_as::<_, HistoricalResultRow>(
        r#"
            SELECT historical.kickoff_time,
                   historical.home_team_id, historical.away_team_id,
                   historical.competition_id, historical.season_id,
                   result.home_goals_90, result.away_goals_90
            FROM football.matches historical
            JOIN football.match_results result ON result.match_id = historical.id
            WHERE historical.id <> $1
              AND historical.kickoff_time < $2
              AND result.finalized_at <= $3
              AND result.created_at <= $3
              AND (historical.home_team_id = $4 OR historical.away_team_id = $4)
            ORDER BY historical.kickoff_time DESC, historical.id DESC
            LIMIT $5
            "#,
    )
    .bind(fixture.id)
    .bind(fixture.kickoff_time)
    .bind(data_cutoff_time)
    .bind(team_id)
    .bind(HISTORY_QUERY_LIMIT)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| map_history(row, fixture, team_id, is_home))
        .collect())
}

fn map_history(
    row: HistoricalResultRow,
    fixture: &MatchRecord,
    team_id: Uuid,
    is_home: bool,
) -> HistoricalMatch {
    let team_was_home = row.home_team_id == team_id;
    let home_goals = row.home_goals_90 as f64;
    let away_goals = row.away_goals_90 as f64;
    let (goals_for, goals_against) = if team_was_home {
        (home_goals, away_goals)
    } else {
        (away_goals, home_goals)
    };
    let points = if goals_for > goals_against {
        3.0
    } else if (goals_for - goals_against).abs() < f64::EPSILON {
        1.0
    } else {
        0.0
    };
    HistoricalMatch {
        kickoff_time: row.kickoff_time,
        goals_for,
        goals_against,
        points,
        played_at_target_venue: team_was_home == is_home,
        same_competition: fixture.competition_id.is_some()
            && row.competition_id == fixture.competition_id,
        same_season: fixture.season_id.is_some() && row.season_id == fixture.season_id,
    }
}

pub(super) async fn goal_baseline(
    pool: &PgPool,
    competition_id: Option<Uuid>,
    kickoff_time: DateTime<Utc>,
    data_cutoff_time: DateTime<Utc>,
) -> PersistenceResult<GoalBaseline> {
    let row = sqlx::query_as::<_, GoalBaselineRow>(
        r#"
            SELECT AVG(result.home_goals_90::double precision) AS home_goals,
                   AVG(result.away_goals_90::double precision) AS away_goals,
                   COUNT(*)::bigint AS match_count
            FROM football.matches historical
            JOIN football.match_results result ON result.match_id = historical.id
            WHERE historical.kickoff_time < $1
              AND result.finalized_at <= $2
              AND result.created_at <= $2
              AND ($3::uuid IS NULL OR historical.competition_id = $3)
            "#,
    )
    .bind(kickoff_time)
    .bind(data_cutoff_time)
    .bind(competition_id)
    .fetch_one(pool)
    .await?;
    Ok(map_baseline(row))
}

fn map_baseline(row: GoalBaselineRow) -> GoalBaseline {
    GoalBaseline {
        home_goals: finite_or_default(
            row.home_goals.unwrap_or(DEFAULT_GOAL_BASELINE),
            DEFAULT_GOAL_BASELINE,
        ),
        away_goals: finite_or_default(
            row.away_goals.unwrap_or(DEFAULT_GOAL_BASELINE),
            DEFAULT_GOAL_BASELINE,
        ),
        match_count: row.match_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::MatchStatus;

    fn fixture() -> MatchRecord {
        MatchRecord {
            id: Uuid::from_u128(1),
            external_key: "HISTORY".into(),
            competition_id: Some(Uuid::from_u128(2)),
            competition_name: None,
            season_id: Some(Uuid::from_u128(3)),
            stage_id: None,
            round_id: None,
            home_team_id: Uuid::from_u128(4),
            home_team_name: "Home".into(),
            away_team_id: Uuid::from_u128(5),
            away_team_name: "Away".into(),
            kickoff_time: DateTime::parse_from_rfc3339("2026-10-01T12:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            status: MatchStatus::Scheduled,
            venue: None,
        }
    }

    #[test]
    fn typed_history_mapping_keeps_team_perspective_venue_and_scope() {
        let mut fixture = fixture();
        for (home_goals, away_goals, expected_points) in [(2, 0, 0.0), (1, 1, 1.0), (0, 2, 3.0)] {
            let row = HistoricalResultRow {
                kickoff_time: fixture.kickoff_time - chrono::Duration::days(1),
                home_team_id: fixture.home_team_id,
                competition_id: fixture.competition_id,
                season_id: fixture.season_id,
                home_goals_90: home_goals,
                away_goals_90: away_goals,
            };
            let mapped = map_history(row, &fixture, fixture.away_team_id, false);
            assert_eq!(mapped.goals_for, f64::from(away_goals));
            assert_eq!(mapped.goals_against, f64::from(home_goals));
            assert_eq!(mapped.points, expected_points);
            assert!(mapped.played_at_target_venue && mapped.same_competition && mapped.same_season);
            assert_eq!(
                mapped.kickoff_time,
                fixture.kickoff_time - chrono::Duration::days(1)
            );
        }
        fixture.competition_id = None;
        fixture.season_id = None;
        let row = HistoricalResultRow {
            kickoff_time: fixture.kickoff_time,
            home_team_id: fixture.home_team_id,
            competition_id: None,
            season_id: None,
            home_goals_90: 2,
            away_goals_90: 0,
        };
        let mapped = map_history(row, &fixture, fixture.home_team_id, false);
        assert_eq!(mapped.points, 3.0);
        assert!(!mapped.played_at_target_venue && !mapped.same_competition && !mapped.same_season);
    }

    #[test]
    fn typed_baseline_mapping_keeps_default_and_non_finite_policy() {
        for (home, away, count, expected_home, expected_away) in [
            (None, None, 0, 1.15, 1.15),
            (Some(f64::NAN), Some(f64::INFINITY), 2, 1.15, 1.15),
            (Some(0.0), Some(1.7), 40, 0.0, 1.7),
        ] {
            let baseline = map_baseline(GoalBaselineRow {
                home_goals: home,
                away_goals: away,
                match_count: count,
            });
            assert_eq!(baseline.home_goals, expected_home);
            assert_eq!(baseline.away_goals, expected_away);
            assert_eq!(baseline.match_count, count);
        }
    }
}
