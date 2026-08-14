use super::{map_stage_row, StageRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::StageRecord;
use uuid::Uuid;

pub(super) async fn read_stage(
    store: &PostgresStore,
    id: Uuid,
) -> PersistenceResult<StageRecord> {
    let row = sqlx::query_as::<_, StageRow>(
        r#"
        SELECT
            st.id, st.season_id, s.name AS season_name,
            s.competition_id, c.name AS competition_name,
            st.code, st.name, st.stage_kind, st.sequence_no
        FROM football.competition_stages st
        JOIN football.seasons s ON s.id = st.season_id
        JOIN football.competitions c ON c.id = s.competition_id
        WHERE st.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&store.pool)
    .await?;
    map_stage_row(row)
}
