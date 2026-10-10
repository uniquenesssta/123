use crate::{GatewayConfig, GatewayError, GatewayErrorCategory, ModelPricing};

pub(crate) fn pricing_for_model<'a>(
    config: &'a GatewayConfig,
    model: &str,
) -> Option<&'a ModelPricing> {
    config.budget.model_pricing.get(model).or_else(|| {
        config
            .budget
            .model_pricing
            .iter()
            .filter(|(configured, _)| {
                model == configured.as_str()
                    || model.starts_with(&format!("{}-", configured.as_str()))
            })
            .max_by_key(|(configured, _)| configured.len())
            .map(|(_, pricing)| pricing)
    })
}

pub(crate) fn budget_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::BudgetExceeded,
        message,
        false,
        "调整预算配置或等待下一预算周期；不得绕过预算生成伪完成状态",
    )
}
