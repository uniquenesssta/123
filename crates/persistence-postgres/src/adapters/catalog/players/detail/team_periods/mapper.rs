use super::row::PlayerTeamPeriodRow;
use football_domain::PlayerTeamPeriodRecord;

pub(super) fn map_player_team_period(row: PlayerTeamPeriodRow) -> PlayerTeamPeriodRecord {
    PlayerTeamPeriodRecord {
        id: row.id,
        player_id: row.player_id,
        team_id: row.team_id,
        team_name: row.team_name,
        season_id: row.season_id,
        season_name: row.season_name,
        squad_number: row.squad_number,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        registration_status: row.registration_status,
    }
}
