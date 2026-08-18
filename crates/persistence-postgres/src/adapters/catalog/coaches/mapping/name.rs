use crate::PersistenceResult;
use football_domain::CoachNameRecord;
use sqlx::Row;

pub(crate) fn coach_name_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<CoachNameRecord> {
    Ok(CoachNameRecord {
        id: row.try_get("id")?,
        coach_id: row.try_get("coach_id")?,
        name: row.try_get("name")?,
        normalized_name: row.try_get("normalized_name")?,
        language_code: row.try_get("language_code")?,
        is_primary: row.try_get("is_primary")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
    })
}
