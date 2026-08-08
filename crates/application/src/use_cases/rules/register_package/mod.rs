use super::package_validation::{validate_parameter_identity, validate_rule_package_shape};
use crate::{
    model_registry::ModelRegistry,
    ports::rules::{RulePackagePort, RuleRoutingPort},
    ApplicationError, ApplicationResult,
};
use football_domain::{RulePackageDraft, RulePackageSummary};

pub(crate) async fn execute<P>(
    registry: &ModelRegistry,
    port: &P,
    draft: RulePackageDraft,
) -> ApplicationResult<RulePackageSummary>
where
    P: RulePackagePort + RuleRoutingPort + ?Sized,
{
    validate_rule_package_shape(&draft)?;
    let model = registry
        .get(&draft.routing.model_id)
        .ok_or_else(|| ApplicationError::ModelNotFound(draft.routing.model_id.clone()))?;
    let descriptor = model.descriptor();
    if !descriptor
        .supported_competitions
        .contains(&draft.competition_profile.competition_kind)
    {
        return Err(ApplicationError::Validation(format!(
            "模型入口 {} 不支持赛事类型 {}",
            draft.routing.model_id,
            draft.competition_profile.competition_kind.as_str()
        )));
    }
    model
        .validate_parameters(&draft.parameters)
        .map_err(|error| ApplicationError::Model(error.to_string()))?;
    validate_parameter_identity(&draft)?;

    let summary = port.register_rule_package(&descriptor, &draft).await?;
    if draft.routing.activate_as_type_default {
        port.ensure_type_default_binding(
            summary.id,
            draft.competition_profile.competition_kind,
            draft.routing.priority,
            &format!("{} 类型默认", draft.display_name),
        )
        .await?;
    }
    Ok(summary)
}
