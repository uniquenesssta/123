use super::row::SeasonTeamMembershipRow;
use football_domain::SeasonTeamMembershipOption;

pub(super) fn map_membership_option(row: SeasonTeamMembershipRow) -> SeasonTeamMembershipOption {
    SeasonTeamMembershipOption {
        season_id: row.season_id,
        team_id: row.team_id,
        registration_status: row.registration_status,
    }
}
