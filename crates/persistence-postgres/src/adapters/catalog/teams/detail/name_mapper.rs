use super::name_row::TeamNameRow;
use football_domain::TeamNameRecord;

pub(super) fn map_team_name(row: TeamNameRow) -> TeamNameRecord {
    TeamNameRecord {
        id: row.id,
        team_id: row.team_id,
        name: row.name,
        normalized_name: row.normalized_name,
        language_code: row.language_code,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
    }
}
