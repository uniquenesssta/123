use super::{mapper::external_entity_id_from_row, validation::validate_external_id};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{ExternalEntityIdDraft, ExternalEntityIdRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_external_entity_id(
        &self,
        draft: &ExternalEntityIdDraft,
    ) -> PersistenceResult<ExternalEntityIdRecord> {
        let validated = validate_external_id(draft)?;
        let row = sqlx::query(
            r#"
            WITH inserted AS (
                INSERT INTO football.external_entity_ids (
                    id, provider_id, entity_type, entity_id, external_id, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (provider_id, entity_type, external_id) DO UPDATE SET
                    entity_id = EXCLUDED.entity_id,
                    metadata = football.external_entity_ids.metadata || EXCLUDED.metadata
                RETURNING *
            )
            SELECT inserted.id, inserted.provider_id,
                   provider.name AS provider_name, inserted.entity_type,
                   inserted.entity_id, inserted.external_id, inserted.metadata
            FROM inserted
            JOIN catalog.data_providers provider ON provider.id = inserted.provider_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.provider_id)
        .bind(validated.entity_type)
        .bind(draft.entity_id)
        .bind(validated.external_id)
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        external_entity_id_from_row(&row)
    }
}
