use crate::PersistenceResult;
use football_domain::MonthlyDataGapRow;
use sqlx::Row;

pub(super) fn monthly_gap_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<MonthlyDataGapRow> {
    Ok(MonthlyDataGapRow {
        entity_type: row.try_get("entity_type")?,
        entity_id: row.try_get("entity_id")?,
        entity_name: row.try_get("entity_name")?,
        missing_field: row.try_get("missing_field")?,
        last_observed_at: row.try_get("last_observed_at")?,
        stale_days: row.try_get("stale_days")?,
        priority: row.try_get("priority")?,
        recommended_action: row.try_get("recommended_action")?,
    })
}
