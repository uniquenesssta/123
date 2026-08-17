use crate::{
    adapters::catalog::availability::{map_player_availability, PlayerAvailabilityRow},
    PersistenceResult, PostgresStore,
};
use football_domain::PlayerAvailabilityRecord;
use uuid::Uuid;

pub(in crate::adapters::catalog::players::detail) async fn read_availability(
    store: &PostgresStore,
    player_id: Uuid,
) -> PersistenceResult<Vec<PlayerAvailabilityRecord>> {
    let rows = sqlx::query_as::<_, PlayerAvailabilityRow>(
        r#"
        SELECT
            availability.id, availability.player_id, availability.team_id,
            team.canonical_name AS team_name, availability.competition_id,
            availability.status, availability.reason, availability.confidence,
            availability.valid_from, availability.valid_to, availability.created_at
        FROM football.player_availability availability
        LEFT JOIN football.teams team ON team.id = availability.team_id
        WHERE availability.player_id = $1
        ORDER BY availability.valid_from DESC, availability.created_at DESC
        LIMIT 100
        "#,
    )
    .bind(player_id)
    .fetch_all(&store.pool)
    .await?;
    rows.into_iter().map(map_player_availability).collect()
}
