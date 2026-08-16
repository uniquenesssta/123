use super::{mapper::map_player_position, row::PlayerPositionRow};
use crate::PersistenceResult;
use football_domain::PlayerPositionRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(in crate::adapters::catalog::players::detail) async fn read_positions(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Vec<PlayerPositionRecord>> {
    let rows = sqlx::query_as::<_, PlayerPositionRow>(
        r#"
        SELECT
            assignment.id, assignment.player_id, assignment.position_code,
            position.name AS position_name, position.position_group,
            assignment.proficiency, assignment.default_role_code, assignment.is_primary,
            assignment.valid_from, assignment.valid_to
        FROM football.player_positions assignment
        JOIN football.positions position ON position.code = assignment.position_code
        WHERE assignment.player_id = $1
        ORDER BY assignment.is_primary DESC, assignment.proficiency DESC,
                 assignment.valid_from DESC NULLS LAST
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_player_position).collect())
}
