use crate::PersistenceResult;
use uuid::Uuid;

pub(crate) async fn entity_label(
    pool: &sqlx::PgPool,
    entity_type: &str,
    id: Uuid,
) -> PersistenceResult<Option<String>> {
    let query = match entity_type {
        "team" => "SELECT canonical_name FROM football.teams WHERE id=$1",
        "player" => "SELECT canonical_name FROM football.players WHERE id=$1",
        "coach" => "SELECT canonical_name FROM football.coaches WHERE id=$1",
        _ => unreachable!(),
    };
    Ok(sqlx::query_scalar(query)
        .bind(id)
        .fetch_optional(pool)
        .await?)
}
