use super::row::PlayerNameRow;
use football_domain::PlayerNameRecord;

pub(in crate::adapters::catalog::players) fn map_player_name(
    row: PlayerNameRow,
) -> PlayerNameRecord {
    PlayerNameRecord {
        id: row.id,
        player_id: row.player_id,
        name: row.name,
        normalized_name: row.normalized_name,
        language_code: row.language_code,
        is_primary: row.is_primary,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
    }
}
