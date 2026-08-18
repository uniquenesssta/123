use super::{mapper::data_provider_from_row, validation::validate_provider};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{DataProviderDraft, DataProviderRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_data_provider(
        &self,
        draft: &DataProviderDraft,
    ) -> PersistenceResult<DataProviderRecord> {
        let validated = validate_provider(draft)?;
        let generated_id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO catalog.data_providers (
                id, code, name, provider_type, base_url, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (code) DO UPDATE SET
                name = EXCLUDED.name,
                provider_type = EXCLUDED.provider_type,
                base_url = EXCLUDED.base_url,
                metadata = catalog.data_providers.metadata || EXCLUDED.metadata,
                is_active = true,
                updated_at = now()
            RETURNING id, code, name, provider_type, base_url, is_active
            "#,
        )
        .bind(generated_id)
        .bind(&validated.code)
        .bind(validated.name)
        .bind(validated.provider_type)
        .bind(validated.base_url)
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        data_provider_from_row(&row)
    }
}
