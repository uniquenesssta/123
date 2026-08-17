use super::{mapper::map_dynamic_tag_definition, row::DynamicTagDefinitionRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::PlayerDynamicTagDefinitionRecord;

impl PostgresStore {
    pub async fn list_dynamic_tag_definitions(
        &self,
    ) -> PersistenceResult<Vec<PlayerDynamicTagDefinitionRecord>> {
        let rows = sqlx::query_as::<_, DynamicTagDefinitionRow>(
            r#"
            SELECT code, name, category, minimum_value, maximum_value,
                   default_value, default_ttl_hours, is_multiplier, description
            FROM feature.player_dynamic_tag_definitions
            ORDER BY category, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(map_dynamic_tag_definition).collect())
    }
}
