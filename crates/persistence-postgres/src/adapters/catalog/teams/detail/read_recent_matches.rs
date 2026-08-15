use super::{recent_match_mapper::map_team_recent_match, recent_match_row::TeamRecentMatchRow};
use crate::PersistenceResult;
use football_domain::TeamRecentMatch;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_recent_matches(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<Vec<TeamRecentMatch>> {
    let rows = sqlx::query_as::<_, TeamRecentMatchRow>(
        r#"
        SELECT fixture.id AS match_id,
               CASE WHEN fixture.home_team_id = $1 THEN fixture.away_team_id ELSE fixture.home_team_id END AS opponent_team_id,
               CASE WHEN fixture.home_team_id = $1 THEN away.canonical_name ELSE home.canonical_name END AS opponent_team_name,
               fixture.kickoff_time,
               CASE WHEN fixture.home_team_id = $1 THEN 'home' ELSE 'away' END AS venue_side,
               fixture.status,
               CASE WHEN result.match_id IS NULL THEN NULL
                    WHEN fixture.home_team_id = $1 THEN result.home_goals_90 ELSE result.away_goals_90 END AS goals_for,
               CASE WHEN result.match_id IS NULL THEN NULL
                    WHEN fixture.home_team_id = $1 THEN result.away_goals_90 ELSE result.home_goals_90 END AS goals_against
        FROM football.matches fixture
        JOIN football.teams home ON home.id = fixture.home_team_id
        JOIN football.teams away ON away.id = fixture.away_team_id
        LEFT JOIN football.match_results result ON result.match_id = fixture.id
        WHERE fixture.home_team_id = $1 OR fixture.away_team_id = $1
        ORDER BY fixture.kickoff_time DESC, fixture.id DESC
        LIMIT 20
        "#,
    )
    .bind(team_id)
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(map_team_recent_match).collect()
}
