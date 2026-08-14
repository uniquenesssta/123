use crate::PersistenceResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn delete_external_entity_ids(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> PersistenceResult<()> {
    sqlx::query(
        "DELETE FROM football.external_entity_ids WHERE entity_type = 'competition' AND entity_id = $1",
    )
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
