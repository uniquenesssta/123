use crate::PersistenceResult;
use football_domain::TeamCoachPeriodRecord;
use sqlx::Row;

pub(crate) fn team_coach_period_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<TeamCoachPeriodRecord> {
    Ok(TeamCoachPeriodRecord {
        id: row.try_get("id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        coach_id: row.try_get("coach_id")?,
        coach_name: row.try_get("coach_name")?,
        role: row.try_get("role")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        is_interim: row.try_get("is_interim")?,
        confidence: row.try_get("confidence")?,
    })
}
