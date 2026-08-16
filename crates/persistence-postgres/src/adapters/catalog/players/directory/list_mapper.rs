use super::list_row::PlayerListRow;
use crate::{
    adapters::catalog::players::value_mapping::{
        availability_status, player_status, preferred_foot,
    },
    PersistenceResult,
};
use football_domain::PlayerListItem;

pub(super) fn map_player_list_item(row: PlayerListRow) -> PersistenceResult<PlayerListItem> {
    Ok(PlayerListItem {
        id: row.id,
        canonical_name: row.canonical_name,
        localized_name: row.localized_name,
        alternate_name: row.alternate_name,
        normalized_name: row.normalized_name,
        date_of_birth: row.date_of_birth,
        nationality_code: row.nationality_code,
        preferred_foot: preferred_foot(&row.preferred_foot)?,
        status: player_status(&row.status)?,
        current_team_id: row.current_team_id,
        current_team_name: row.current_team_name,
        primary_position_code: row.primary_position_code,
        primary_role_code: row.primary_role_code,
        position_role_map: row.position_role_map,
        availability_status: row
            .availability_status
            .as_deref()
            .map(availability_status)
            .transpose()?,
        availability_reason: row.availability_reason,
        availability_confidence: row.availability_confidence,
        availability_valid_to: row.availability_valid_to,
        availability_competition_name: row.availability_competition_name,
        ability_average: row.ability_average,
        ability_confidence: row.ability_confidence,
        ability_dimension_count: row.ability_dimension_count,
    })
}
