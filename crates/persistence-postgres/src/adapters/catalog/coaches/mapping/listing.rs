use crate::PersistenceResult;
use football_domain::CoachListItem;
use sqlx::Row;

pub(crate) fn coach_list_item_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<CoachListItem> {
    Ok(CoachListItem {
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        nationality_code: row.try_get("nationality_code")?,
        status: row.try_get("status")?,
        current_team_id: row.try_get("current_team_id")?,
        current_team_name: row.try_get("current_team_name")?,
        current_role: row.try_get("current_role")?,
    })
}
