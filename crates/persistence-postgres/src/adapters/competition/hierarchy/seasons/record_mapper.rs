use super::SeasonRow;
use football_domain::SeasonRecord;

pub(super) fn map_season_row(row: SeasonRow) -> SeasonRecord {
    SeasonRecord {
        id: row.id,
        competition_id: row.competition_id,
        competition_name: row.competition_name,
        name: row.name,
        starts_on: row.starts_on,
        ends_on: row.ends_on,
        status: row.status,
    }
}
