use crate::{PersistenceError, PersistenceResult};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn lock_active_competition(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
) -> PersistenceResult<String> {
    sqlx::query_scalar(
        "SELECT name FROM football.competitions WHERE id = $1 AND is_active = true FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("赛事不存在或已经删除".to_string()))
}
