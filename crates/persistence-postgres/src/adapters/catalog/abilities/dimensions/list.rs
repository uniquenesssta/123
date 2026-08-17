use super::{mapper::map_ability_dimension, row::AbilityDimensionRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::AbilityDimensionRecord;

impl PostgresStore {
    pub async fn list_ability_dimensions(&self) -> PersistenceResult<Vec<AbilityDimensionRecord>> {
        let rows = sqlx::query_as::<_, AbilityDimensionRow>(
            r#"
            SELECT code, name, category, minimum_value, maximum_value, description
            FROM feature.player_ability_dimensions
            ORDER BY category, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(map_ability_dimension).collect())
    }
}
