use super::row::DynamicTagDefinitionRow;
use football_domain::PlayerDynamicTagDefinitionRecord;

pub(super) fn map_dynamic_tag_definition(
    row: DynamicTagDefinitionRow,
) -> PlayerDynamicTagDefinitionRecord {
    PlayerDynamicTagDefinitionRecord {
        code: row.code,
        name: row.name,
        category: row.category,
        minimum_value: row.minimum_value,
        maximum_value: row.maximum_value,
        default_value: row.default_value,
        default_ttl_hours: row.default_ttl_hours,
        is_multiplier: row.is_multiplier,
        description: row.description,
    }
}
