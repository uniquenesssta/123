use super::{name_mapper::map_team_name, name_row::TeamNameRow};
use crate::PersistenceResult;
use football_domain::TeamNameRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_names(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<Vec<TeamNameRecord>> {
    let rows = sqlx::query_as::<_, TeamNameRow>(
        r#"
        SELECT id, team_id, name, normalized_name, language_code, valid_from, valid_to
        FROM football.team_names
        WHERE team_id = $1
        ORDER BY valid_from DESC NULLS LAST, name, id
        "#,
    )
    .bind(team_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_team_name).collect())
}
