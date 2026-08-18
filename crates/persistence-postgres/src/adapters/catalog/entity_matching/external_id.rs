use crate::PersistenceResult;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn matching_entity_ids(
    pool: &PgPool,
    provider_id: Uuid,
    entity_type: &str,
    external_id: &str,
) -> PersistenceResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT entity_id FROM football.external_entity_ids WHERE provider_id=$1 AND entity_type=$2 AND external_id=$3",
    )
    .bind(provider_id)
    .bind(entity_type)
    .bind(external_id)
    .fetch_all(pool)
    .await?)
}
