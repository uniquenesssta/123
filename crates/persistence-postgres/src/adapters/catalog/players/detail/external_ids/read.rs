use super::{mapper::map_external_entity_id, row::ExternalEntityIdRow};
use crate::PersistenceResult;
use football_domain::ExternalEntityIdRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(in crate::adapters::catalog::players::detail) async fn read_external_ids(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Vec<ExternalEntityIdRecord>> {
    let rows = sqlx::query_as::<_, ExternalEntityIdRow>(
        r#"
        SELECT external.id, external.provider_id, provider.name AS provider_name,
               external.entity_type, external.entity_id, external.external_id,
               external.metadata
        FROM football.external_entity_ids external
        JOIN catalog.data_providers provider ON provider.id = external.provider_id
        WHERE external.entity_type = 'player' AND external.entity_id = $1
        ORDER BY provider.name, external.external_id
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_external_entity_id).collect())
}
