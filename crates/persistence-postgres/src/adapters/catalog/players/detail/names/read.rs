use super::{mapper::map_player_name, row::PlayerNameRow};
use crate::PersistenceResult;
use football_domain::PlayerNameRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_names(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Vec<PlayerNameRecord>> {
    let rows = sqlx::query_as::<_, PlayerNameRow>(
        r#"
        SELECT id, player_id, name, normalized_name, language_code,
               is_primary, valid_from, valid_to
        FROM football.player_names
        WHERE player_id = $1
        ORDER BY is_primary DESC, valid_from DESC NULLS LAST, name
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_player_name).collect())
}
