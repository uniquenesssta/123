use crate::{
    adapters::catalog::abilities::observations::{
        map_player_ability_observation, PlayerAbilityObservationRow,
    },
    PersistenceResult,
};
use football_domain::PlayerAbilityObservationRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(in crate::adapters::catalog::players::detail) async fn read_ability_observations(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Vec<PlayerAbilityObservationRecord>> {
    let rows = sqlx::query_as::<_, PlayerAbilityObservationRow>(
        r#"
        SELECT
            observation.id, observation.player_id, observation.dimension_code,
            dimension.name AS dimension_name, observation.context_type,
            observation.context_id, observation.value, observation.confidence,
            observation.sample_size, observation.observed_at,
            observation.effective_from, observation.effective_to,
            observation.calculation_version
        FROM feature.player_ability_observations observation
        JOIN feature.player_ability_dimensions dimension
          ON dimension.code = observation.dimension_code
        WHERE observation.player_id = $1
        ORDER BY observation.observed_at DESC, observation.id DESC
        LIMIT 250
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(map_player_ability_observation)
        .collect())
}
