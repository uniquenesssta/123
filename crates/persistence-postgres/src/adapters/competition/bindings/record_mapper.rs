use football_domain::CompetitionBindingSummary;

use crate::{parse_competition_kind, PersistenceResult};

use super::BindingRow;

pub(in crate::adapters::competition::bindings) fn map_binding_row(
    row: BindingRow,
) -> PersistenceResult<CompetitionBindingSummary> {
    Ok(CompetitionBindingSummary {
        id: row.id,
        binding_name: row.binding_name,
        competition_id: row.competition_id,
        competition_name: row.competition_name,
        season_id: row.season_id,
        stage_id: row.stage_id,
        competition_kind: row
            .competition_kind
            .map(|value| parse_competition_kind(&value))
            .transpose()?,
        rule_package_id: row.rule_package_id,
        rule_package_name: row.rule_package_name,
        model_id: row.model_key,
        priority: row.priority,
        is_active: row.is_active,
        created_at: row.created_at,
    })
}
