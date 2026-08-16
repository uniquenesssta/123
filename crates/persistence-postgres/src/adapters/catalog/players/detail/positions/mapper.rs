use super::row::PlayerPositionRow;
use football_domain::PlayerPositionRecord;

pub(super) fn map_player_position(row: PlayerPositionRow) -> PlayerPositionRecord {
    PlayerPositionRecord {
        id: row.id,
        player_id: row.player_id,
        position_code: row.position_code,
        position_name: row.position_name,
        position_group: row.position_group,
        proficiency: row.proficiency,
        default_role_code: row.default_role_code,
        is_primary: row.is_primary,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
    }
}
