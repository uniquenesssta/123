use crate::PersistenceResult;
use football_domain::PostmatchDriftFindingRecord;
use sqlx::Row;

pub(super) fn drift_finding_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PostmatchDriftFindingRecord> {
    Ok(PostmatchDriftFindingRecord {
        metric_name: row.try_get("metric_name")?,
        baseline_mean: row.try_get("baseline_mean")?,
        current_mean: row.try_get("current_mean")?,
        absolute_delta: row.try_get("absolute_delta")?,
        relative_delta: row.try_get("relative_delta")?,
        severity: row.try_get("severity")?,
        direction: row.try_get("direction")?,
    })
}
