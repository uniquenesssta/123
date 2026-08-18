use crate::PersistenceResult;
use football_domain::DataProviderRecord;
use sqlx::Row;

pub(super) fn data_provider_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<DataProviderRecord> {
    Ok(DataProviderRecord {
        id: row.try_get("id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        provider_type: row.try_get("provider_type")?,
        base_url: row.try_get("base_url")?,
        is_active: row.try_get("is_active")?,
    })
}
