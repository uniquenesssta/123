use crate::{PersistenceResult, PostgresStore};
use football_domain::TeamPlayerPeriodRecord;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub(crate) async fn list_team_player_periods(
        &self,
        team_id: Uuid,
    ) -> PersistenceResult<Vec<TeamPlayerPeriodRecord>> {
        sqlx::query(
            r#"
            SELECT period.id, period.team_id, team.canonical_name AS team_name,
                   period.player_id, player.canonical_name AS player_name,
                   period.season_id, season.name AS season_name, period.squad_number,
                   period.valid_from, period.valid_to, period.registration_status
            FROM football.player_team_periods period
            JOIN football.teams team ON team.id=period.team_id
            JOIN football.players player ON player.id=period.player_id
            LEFT JOIN football.seasons season ON season.id=period.season_id
            WHERE period.team_id=$1
            ORDER BY period.valid_from DESC, period.valid_to DESC NULLS FIRST,
                     player.normalized_name, period.id
            "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(team_player_period_from_row)
        .collect()
    }
}

fn team_player_period_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<TeamPlayerPeriodRecord> {
    Ok(TeamPlayerPeriodRecord {
        id: row.try_get("id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        season_id: row.try_get("season_id")?,
        season_name: row.try_get("season_name")?,
        squad_number: row.try_get("squad_number")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        registration_status: row.try_get("registration_status")?,
    })
}
