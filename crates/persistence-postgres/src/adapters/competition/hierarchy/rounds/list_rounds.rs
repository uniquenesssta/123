use super::{map_round_row, RoundRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::RoundRecord;

impl PostgresStore {
    pub async fn list_rounds(&self) -> PersistenceResult<Vec<RoundRecord>> {
        let rows = sqlx::query_as::<_, RoundRow>(
            r#"
            SELECT
                r.id, r.stage_id, st.name AS stage_name,
                r.code, r.name, r.sequence_no, r.starts_at, r.ends_at
            FROM football.rounds r
            JOIN football.competition_stages st ON st.id = r.stage_id
            JOIN football.seasons s ON s.id = st.season_id
            JOIN football.competitions c ON c.id = s.competition_id
            WHERE c.is_active = true
            ORDER BY st.name, r.sequence_no, r.starts_at NULLS LAST
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(map_round_row).collect())
    }
}
