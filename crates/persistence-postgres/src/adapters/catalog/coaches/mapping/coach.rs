use crate::PersistenceResult;
use football_domain::CoachRecord;
use sqlx::Row;

pub(crate) fn coach_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<CoachRecord> {
    Ok(CoachRecord {
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        nationality_code: row.try_get("nationality_code")?,
        status: row.try_get("status")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
