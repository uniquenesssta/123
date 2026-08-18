use super::mapper::data_provider_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::DataProviderRecord;

impl PostgresStore {
    pub async fn list_data_providers(&self) -> PersistenceResult<Vec<DataProviderRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, code, name, provider_type, base_url, is_active
            FROM catalog.data_providers
            WHERE is_active
            ORDER BY name, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(data_provider_from_row).collect()
    }
}
