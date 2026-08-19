use super::{mapper::map_team_player_period, row::TeamPlayerPeriodRow};
use crate::PersistenceResult;
use football_domain::TeamPlayerPeriodRecord;
use uuid::Uuid;

pub(in crate::adapters::catalog::teams::detail) async fn read_player_periods(
    pool: &sqlx::PgPool,
    team_id: Uuid,
) -> PersistenceResult<Vec<TeamPlayerPeriodRecord>> {
    let rows = sqlx::query_as::<_, TeamPlayerPeriodRow>(
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
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_team_player_period).collect())
}
