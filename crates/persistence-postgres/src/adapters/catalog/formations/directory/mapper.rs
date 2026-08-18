use crate::PersistenceResult;
use football_domain::FormationRecord;
use sqlx::Row;

pub(crate) fn formation_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<FormationRecord> {
    Ok(FormationRecord {
        id: row.try_get("id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        line_structure: row.try_get("line_structure")?,
        slot_definition: row.try_get("slot_definition")?,
        is_builtin: row.try_get("is_builtin")?,
        is_active: row.try_get("is_active")?,
        sort_order: row.try_get("sort_order")?,
        metadata: row.try_get("metadata")?,
    })
}
