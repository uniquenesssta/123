use crate::{PersistenceError, PersistenceResult};
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn entity_exists(
    pool: &PgPool,
    entity_type: &str,
    id: Uuid,
) -> PersistenceResult<bool> {
    let query = match entity_type {
        "team" => "SELECT EXISTS(SELECT 1 FROM football.teams WHERE id=$1)",
        "player" => "SELECT EXISTS(SELECT 1 FROM football.players WHERE id=$1)",
        "coach" => "SELECT EXISTS(SELECT 1 FROM football.coaches WHERE id=$1)",
        other => {
            return Err(PersistenceError::InvalidState(format!(
                "不支持的实体类型：{other}"
            )))
        }
    };
    Ok(sqlx::query_scalar(query).bind(id).fetch_one(pool).await?)
}
