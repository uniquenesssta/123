use super::row::AbilityDimensionRow;
use football_domain::AbilityDimensionRecord;

pub(super) fn map_ability_dimension(row: AbilityDimensionRow) -> AbilityDimensionRecord {
    AbilityDimensionRecord {
        code: row.code,
        name: row.name,
        category: row.category,
        minimum_value: row.minimum_value,
        maximum_value: row.maximum_value,
        description: row.description,
    }
}
