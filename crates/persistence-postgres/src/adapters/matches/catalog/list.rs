use super::mapping::match_record_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::MatchRecord;

impl PostgresStore {
    pub async fn list_upcoming_matches(&self, limit: u32) -> PersistenceResult<Vec<MatchRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                fixture.id, fixture.external_key,
                fixture.competition_id, competition.name AS competition_name,
                fixture.season_id, fixture.stage_id, fixture.round_id,
                fixture.home_team_id, home.canonical_name AS home_team_name,
                fixture.away_team_id, away.canonical_name AS away_team_name,
                fixture.kickoff_time, fixture.status, fixture.venue
            FROM football.matches fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            WHERE fixture.status IN ('scheduled', 'live')
            ORDER BY fixture.kickoff_time, fixture.id
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 250)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(match_record_from_row).collect()
    }

    pub async fn list_managed_matches(&self, limit: u32) -> PersistenceResult<Vec<MatchRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                fixture.id, fixture.external_key,
                fixture.competition_id, competition.name AS competition_name,
                fixture.season_id, fixture.stage_id, fixture.round_id,
                fixture.home_team_id, home.canonical_name AS home_team_name,
                fixture.away_team_id, away.canonical_name AS away_team_name,
                fixture.kickoff_time, fixture.status, fixture.venue
            FROM football.matches fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            ORDER BY fixture.kickoff_time DESC, fixture.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(match_record_from_row).collect()
    }
}
