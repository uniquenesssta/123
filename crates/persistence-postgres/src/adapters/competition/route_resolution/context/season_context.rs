use super::record_row::SeasonContextRow;
use crate::{PersistenceError, PersistenceResult};
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_season_context(
    pool: &PgPool,
    season_id: Uuid,
) -> PersistenceResult<SeasonContextRow> {
    sqlx::query_as::<_, SeasonContextRow>(
        r#"
        SELECT c.id AS competition_id, s.id AS season_id, c.competition_kind
        FROM football.seasons s
        JOIN football.competitions c ON c.id = s.competition_id
        WHERE s.id = $1 AND c.is_active = true
        "#,
    )
    .bind(season_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        PersistenceError::InvalidState(format!("赛季不存在或所属赛事已停用：{season_id}"))
    })
}
