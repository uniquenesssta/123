use super::mapper::formation_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::FormationRecord;

impl PostgresStore {
    pub async fn list_formations(
        &self,
        active_only: bool,
    ) -> PersistenceResult<Vec<FormationRecord>> {
        let rows = sqlx::query(
            r#"
                SELECT id, code, name, line_structure, slot_definition,
                       is_builtin, is_active, sort_order, metadata
                FROM football.formations
                WHERE NOT $1 OR is_active
                ORDER BY sort_order, code, id
                "#,
        )
        .bind(active_only)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(formation_from_row).collect()
    }
}
