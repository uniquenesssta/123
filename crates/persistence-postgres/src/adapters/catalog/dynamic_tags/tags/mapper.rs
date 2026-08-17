use super::row::PlayerDynamicTagRow;
use football_domain::PlayerDynamicTagRecord;

pub(in crate::adapters::catalog::dynamic_tags) fn map_player_dynamic_tag(
    row: PlayerDynamicTagRow,
) -> PlayerDynamicTagRecord {
    PlayerDynamicTagRecord {
        id: row.id,
        player_id: row.player_id,
        tag_code: row.tag_code,
        tag_name: row.tag_name,
        category: row.category,
        value: row.value,
        label: row.label,
        confidence: row.confidence,
        observed_at: row.observed_at,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        competition_id: row.competition_id,
        competition_name: row.competition_name,
        position_code: row.position_code,
        opponent_team_id: row.opponent_team_id,
        opponent_team_name: row.opponent_team_name,
        sample_size: row.sample_size,
        source_type: row.source_type,
        calculation_version: row.calculation_version,
        metadata: row.metadata,
    }
}
