use super::{mapper::external_entity_id_from_row, validation::validate_external_id};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{ExternalEntityIdDraft, ExternalEntityIdRecord};
use sqlx::PgConnection;
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_external_entity_id(
        &self,
        draft: &ExternalEntityIdDraft,
    ) -> PersistenceResult<ExternalEntityIdRecord> {
        validate_external_id(draft)?;
        let mut connection = self.pool.acquire().await?;
        write_external_entity_id(&mut connection, draft).await
    }
}

// The same atomic identity rule serves direct calls and the caller-owned import transaction.
pub(crate) async fn write_external_entity_id(
    connection: &mut PgConnection,
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
                    metadata = football.external_entity_ids.metadata || EXCLUDED.metadata
                WHERE football.external_entity_ids.entity_id = EXCLUDED.entity_id
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
    .fetch_optional(connection)
    .await?
    .ok_or_else(|| {
        PersistenceError::InvalidState(
            "该外部 ID 已绑定到另一条数据库记录，禁止自动改绑".to_string(),
        )
    })?;
    external_entity_id_from_row(&row)
}
