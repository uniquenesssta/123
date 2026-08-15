use super::record_row::StageContextRow;
use crate::{PersistenceError, PersistenceResult};
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_stage_context(
    pool: &PgPool,
    stage_id: Uuid,
) -> PersistenceResult<StageContextRow> {
    sqlx::query_as::<_, StageContextRow>(
        r#"
        SELECT
            c.id AS competition_id, s.id AS season_id, st.id AS stage_id,
            st.stage_kind
        FROM football.competition_stages st
        JOIN football.seasons s ON s.id = st.season_id
        JOIN football.competitions c ON c.id = s.competition_id
        WHERE st.id = $1 AND c.is_active = true
        "#,
    )
    .bind(stage_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        PersistenceError::InvalidState(format!("赛事阶段不存在或所属赛事已停用：{stage_id}"))
    })
}
