use super::squad_row::TeamSquadRow;
use crate::{PersistenceError, PersistenceResult};
use football_domain::{AvailabilityStatus, TeamSquadPlayer};

pub(super) fn map_team_squad_row(row: TeamSquadRow) -> PersistenceResult<TeamSquadPlayer> {
    Ok(TeamSquadPlayer {
        player_id: row.player_id,
        player_name: row.player_name,
        localized_name: row.localized_name,
        position_code: row.position_code,
        role_code: row.role_code,
        squad_number: row.squad_number,
        registration_status: row.registration_status,
        availability_status: row
            .availability_status
            .as_deref()
            .map(parse_availability)
            .transpose()?,
        ability_average: row.ability_average,
    })
}

fn parse_availability(value: &str) -> PersistenceResult<AvailabilityStatus> {
    match value {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知球员可用状态：{other}"
        ))),
    }
}
