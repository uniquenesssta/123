use super::RulePackageRow;
use crate::{parse_competition_kind, PersistenceResult};
use football_domain::RulePackageSummary;

pub(super) fn map_rule_package_row(row: RulePackageRow) -> PersistenceResult<RulePackageSummary> {
    Ok(RulePackageSummary {
        id: row.id,
        format_version: row.format_version,
        package_key: row.package_key,
        version: row.version,
        display_name: row.display_name,
        competition_kind: parse_competition_kind(&row.competition_kind)?,
        model_id: row.model_key,
        model_version: row.model_version,
        parameter_version: row.parameter_version,
        priority: row.priority,
        content_sha256: row.content_sha256,
        status: row.status,
        created_at: row.created_at,
    })
}
