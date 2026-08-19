use super::{reference_mapper::map_position_reference, reference_row::PositionReferenceRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::PositionReference;

impl PostgresStore {
    pub async fn list_positions(&self) -> PersistenceResult<Vec<PositionReference>> {
        let rows = sqlx::query_as::<_, PositionReferenceRow>(
            r#"
            SELECT code, name, position_group, sort_order
            FROM football.positions
            ORDER BY sort_order, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(map_position_reference).collect())
    }
}
