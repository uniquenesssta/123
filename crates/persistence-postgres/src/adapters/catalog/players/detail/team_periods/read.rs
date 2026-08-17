use crate::{
    adapters::catalog::players::team_periods::{map_player_team_period, PlayerTeamPeriodRow},
    PersistenceResult,
};
use football_domain::PlayerTeamPeriodRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(in crate::adapters::catalog::players::detail) async fn read_team_periods(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Vec<PlayerTeamPeriodRecord>> {
    let rows = sqlx::query_as::<_, PlayerTeamPeriodRow>(
        r#"
        SELECT
            period.id, period.player_id, period.team_id,
            team.canonical_name AS team_name,
            period.season_id, season.name AS season_name,
            period.squad_number, period.valid_from, period.valid_to,
            period.registration_status
        FROM football.player_team_periods period
        JOIN football.teams team ON team.id = period.team_id
        LEFT JOIN football.seasons season ON season.id = period.season_id
        WHERE period.player_id = $1
        ORDER BY period.valid_from DESC, period.id DESC
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_player_team_period).collect())
}
