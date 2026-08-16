use super::{mapper::map_player_ability_profile, row::PlayerAbilityProfileRow};
use crate::PersistenceResult;
use football_domain::PlayerAbilityProfile;
use sqlx::PgPool;
use uuid::Uuid;

pub(in crate::adapters::catalog::players::detail) async fn read_ability_profile(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Option<PlayerAbilityProfile>> {
    let row = sqlx::query_as::<_, PlayerAbilityProfileRow>(
        r#"
        SELECT player_id, abilities, average_value, average_confidence,
               dimension_count, latest_observed_at, next_expiry_at, updated_at
        FROM feature.player_ability_profiles
        WHERE player_id = $1
          AND (next_expiry_at IS NULL OR next_expiry_at >= now())
        "#,
    )
    .bind(player_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(map_player_ability_profile))
}
