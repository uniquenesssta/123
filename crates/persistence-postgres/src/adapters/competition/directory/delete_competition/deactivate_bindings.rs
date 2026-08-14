use crate::PersistenceResult;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn deactivate_bindings(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> PersistenceResult<()> {
    sqlx::query(
        "UPDATE model.competition_bindings SET is_active = false WHERE competition_id = $1",
    )
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
