use super::row::TeamPlayerPeriodRow;
use football_domain::TeamPlayerPeriodRecord;

pub(super) fn map_team_player_period(row: TeamPlayerPeriodRow) -> TeamPlayerPeriodRecord {
    TeamPlayerPeriodRecord {
        id: row.id,
        team_id: row.team_id,
        team_name: row.team_name,
        player_id: row.player_id,
        player_name: row.player_name,
        season_id: row.season_id,
        season_name: row.season_name,
        squad_number: row.squad_number,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        registration_status: row.registration_status,
    }
}
