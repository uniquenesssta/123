use crate::{ApplicationResult, ModelRegistry};
use football_domain::RulePackageDraft;

/// Startup providers and their matching rule catalog, supplied by the application host.
/// Provider assets remain the host's responsibility; this configuration performs no file loading.
pub struct ApplicationModelRuntime {
    pub registry: ModelRegistry,
    pub default_prediction_request: football_model_api::ModelRequest,
    pub startup_rule_packages: Vec<RulePackageDraft>,
    pub default_rule_package_template: RulePackageDraft,
}

impl ApplicationModelRuntime {
    pub(crate) fn validate(&self) -> ApplicationResult<()> {
        for draft in self
            .startup_rule_packages
            .iter()
            .chain(std::iter::once(&self.default_rule_package_template))
        {
            crate::use_cases::rules::register_package::validate(&self.registry, draft)?;
        }
        let request = &self.default_prediction_request;
        let model = self
            .registry
            .get(&request.identity.model_id)
            .ok_or_else(|| {
                crate::ApplicationError::ModelNotFound(request.identity.model_id.clone())
            })?;
        if !model.supports(&request.context) {
            return Err(crate::ApplicationError::Validation(
                "默认演练请求与提供器支持的赛事类型不匹配".into(),
            ));
        }
        model
            .validate(request)
            .map_err(|error| crate::ApplicationError::Model(error.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
