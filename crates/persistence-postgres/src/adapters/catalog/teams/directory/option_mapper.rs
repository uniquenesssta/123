use super::option_row::TeamOptionRow;
use football_domain::TeamOption;

pub(super) fn map_team_option_row(row: TeamOptionRow) -> TeamOption {
    TeamOption {
        id: row.id,
        canonical_name: row.canonical_name,
        country_code: row.country_code,
        team_type: row.team_type,
    }
}
