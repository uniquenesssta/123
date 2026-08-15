use super::list_row::TeamListRow;
use football_domain::TeamListItem;

pub(super) fn map_team_list_row(row: TeamListRow) -> TeamListItem {
    TeamListItem {
        id: row.id,
        canonical_name: row.canonical_name,
        normalized_name: row.normalized_name,
        country_code: row.country_code,
        team_type: row.team_type,
        current_coach_name: row.current_coach_name,
        is_active: row.is_active,
        current_player_count: row.current_player_count,
        unavailable_player_count: row.unavailable_player_count,
        squad_ability_average: row.squad_ability_average,
        profile_confidence: row.profile_confidence,
    }
}
