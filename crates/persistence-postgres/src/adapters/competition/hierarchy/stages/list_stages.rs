use super::{map_stage_row, StageRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::StageRecord;

impl PostgresStore {
    pub async fn list_stages(&self) -> PersistenceResult<Vec<StageRecord>> {
        let rows = sqlx::query_as::<_, StageRow>(
            r#"
            SELECT
                st.id, st.season_id, s.name AS season_name,
                s.competition_id, c.name AS competition_name,
                st.code, st.name, st.stage_kind, st.sequence_no
            FROM football.competition_stages st
            JOIN football.seasons s ON s.id = st.season_id
            JOIN football.competitions c ON c.id = s.competition_id
            WHERE c.is_active = true
            ORDER BY c.name, s.name, st.sequence_no, st.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(map_stage_row).collect()
    }
}
