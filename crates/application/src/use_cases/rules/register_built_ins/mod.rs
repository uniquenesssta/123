use crate::{
    model_registry::ModelRegistry,
    ports::rules::{RulePackagePort, RuleRoutingPort},
    ApplicationError, ApplicationResult,
};
use football_domain::RulePackageDraft;

pub(crate) async fn execute<P>(
    registry: &ModelRegistry,
    port: &P,
    drafts: &[RulePackageDraft],
) -> ApplicationResult<()>
where
    P: RulePackagePort + RuleRoutingPort + ?Sized,
{
    for draft in drafts {
        let model = registry
            .get(&draft.routing.model_id)
            .ok_or_else(|| ApplicationError::ModelNotFound(draft.routing.model_id.clone()))?;
        model
            .validate_parameters(&draft.parameters)
            .map_err(|error| ApplicationError::Model(error.to_string()))?;
        let summary = port
            .register_rule_package(&model.descriptor(), draft)
            .await?;
        if draft.routing.activate_as_type_default {
            port.ensure_type_default_binding(
                summary.id,
                draft.competition_profile.competition_kind,
                draft.routing.priority,
                &format!("内置默认 · {}", draft.display_name),
            )
            .await?;
        }
    }
    Ok(())
}
