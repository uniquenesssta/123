use crate::PersistenceResult;
use football_domain::ExternalEntityIdRecord;
use sqlx::Row;

pub(crate) fn external_id_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ExternalEntityIdRecord> {
    Ok(ExternalEntityIdRecord {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        provider_name: row.try_get("provider_name")?,
        entity_type: row.try_get("entity_type")?,
        entity_id: row.try_get("entity_id")?,
        external_id: row.try_get("external_id")?,
        metadata: row.try_get("metadata")?,
    })
}
