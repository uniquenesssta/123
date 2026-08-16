use super::row::PlayerAbilityObservationRow;
use football_domain::PlayerAbilityObservationRecord;

pub(super) fn map_player_ability_observation(
    row: PlayerAbilityObservationRow,
) -> PlayerAbilityObservationRecord {
    PlayerAbilityObservationRecord {
        id: row.id,
        player_id: row.player_id,
        dimension_code: row.dimension_code,
        dimension_name: row.dimension_name,
        context_type: row.context_type,
        context_id: row.context_id,
        value: row.value,
        confidence: row.confidence,
        sample_size: row.sample_size,
        observed_at: row.observed_at,
        effective_from: row.effective_from,
        effective_to: row.effective_to,
        calculation_version: row.calculation_version,
    }
}
