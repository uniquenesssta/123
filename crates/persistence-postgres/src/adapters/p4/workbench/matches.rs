use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::P4MatchWorkspace;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_p4_match_workspace(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<P4MatchWorkspace> {
        let row = sqlx::query(
            r#"
            SELECT fixture.id, fixture.external_key, fixture.kickoff_time,
                   home.canonical_name AS home_team_name,
                   away.canonical_name AS away_team_name,
                   competition.name AS competition_name
            FROM football.matches fixture
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            WHERE fixture.id = $1
            "#,
        )
        .bind(match_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
        let tasks = self.list_p4_freeze_tasks(Some(match_id), 100).await?;
        Ok(P4MatchWorkspace {
            match_id: row.try_get("id")?,
            match_key: row.try_get("external_key")?,
            home_team_name: row.try_get("home_team_name")?,
            away_team_name: row.try_get("away_team_name")?,
            kickoff_at: row.try_get("kickoff_time")?,
            competition_name: row.try_get("competition_name")?,
            tasks,
        })
    }
}
