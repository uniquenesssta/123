use super::row::PlayerAbilityProfileRow;
use football_domain::PlayerAbilityProfile;

pub(super) fn map_player_ability_profile(row: PlayerAbilityProfileRow) -> PlayerAbilityProfile {
    PlayerAbilityProfile {
        player_id: row.player_id,
        abilities: row.abilities,
        average_value: row.average_value,
        average_confidence: row.average_confidence,
        dimension_count: row.dimension_count,
        latest_observed_at: row.latest_observed_at,
        next_expiry_at: row.next_expiry_at,
        updated_at: row.updated_at,
    }
}
