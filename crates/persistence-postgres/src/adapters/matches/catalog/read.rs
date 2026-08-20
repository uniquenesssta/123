use super::mapping::match_record_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::MatchRecord;
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_match(&self, match_id: Uuid) -> PersistenceResult<MatchRecord> {
        let row = sqlx::query(
            r#"SELECT match.id,match.external_key,match.competition_id,competition.name AS competition_name,
                      match.season_id,match.stage_id,match.round_id,match.home_team_id,
                      home.canonical_name AS home_team_name,match.away_team_id,
                      away.canonical_name AS away_team_name,match.kickoff_time,match.status,match.venue
               FROM football.matches match
               LEFT JOIN football.competitions competition ON competition.id=match.competition_id
               JOIN football.teams home ON home.id=match.home_team_id
               JOIN football.teams away ON away.id=match.away_team_id
               WHERE match.id=$1"#,
        ).bind(match_id).fetch_one(&self.pool).await?;
        match_record_from_row(&row)
    }
}
