use super::record_row::RouteRow;
use crate::PersistenceResult;
use football_domain::{CompetitionProfile, RouteDecision, RouteRequest, RouteSource};
use serde_json::json;

pub(super) fn binding_source(row: &RouteRow) -> RouteSource {
    if row.stage_id.is_some() {
        RouteSource::StageBinding
    } else if row.season_id.is_some() {
        RouteSource::SeasonBinding
    } else if row.competition_id.is_some() {
        RouteSource::CompetitionBinding
    } else {
        RouteSource::CompetitionKindDefault
    }
}

pub(super) fn map_route_row(
    row: RouteRow,
    source: RouteSource,
    request: &RouteRequest,
) -> PersistenceResult<RouteDecision> {
    let competition_profile: CompetitionProfile = serde_json::from_value(row.profile)?;
    let routing = serde_json::from_value(row.routing)?;
    let reason = json!({
        "source": &source,
        "binding_id": row.binding_id,
        "rule_package_id": row.rule_package_id,
        "package_key": &row.package_key,
        "package_version": &row.package_version,
        "preferred_model_family": request.preferred_model_family.as_deref(),
        "preferred_model_id": request.preferred_model_id.as_deref(),
        "competition_id": request.competition_id,
        "season_id": request.season_id,
        "stage_id": request.stage_id,
        "competition_kind": request.competition_kind,
        "priority": row.priority,
    });
    Ok(RouteDecision {
        source,
        binding_id: row.binding_id,
        rule_package_id: row.rule_package_id,
        package_key: row.package_key,
        package_version: row.package_version,
        package_display_name: row.package_display_name,
        model_id: row.model_key,
        model_version_id: row.model_version_id,
        model_version: row.model_version,
        parameter_set_id: row.parameter_set_id,
        parameter_version: row.parameter_version,
        competition_profile_id: row.competition_profile_id,
        parameters: row.parameters,
        routing,
        competition_profile,
        feature_requirements: row.feature_requirements,
        output_contract: row.output_contract,
        priority: row.priority,
        reason,
    })
}
