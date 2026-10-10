use crate::budget::{budget_error, pricing_for_model};
use crate::resilience::attempt_limit;
use crate::{GatewayConfig, GatewayError, GatewayRequest};

pub(crate) fn check_budget(
    config: &GatewayConfig,
    request: &GatewayRequest,
    primary_model: &str,
) -> Result<(), GatewayError> {
    if !request.daily_spend_usd.is_finite()
        || request.daily_spend_usd < 0.0
        || !request.monthly_spend_usd.is_finite()
        || request.monthly_spend_usd < 0.0
    {
        return Err(budget_error("数据库返回了无效的OpenAI预算用量"));
    }
    let mut models = vec![primary_model];
    if let Some(fallback) = config.fallback_model.as_deref() {
        if !models.contains(&fallback) {
            models.push(fallback);
        }
    }
    let attempts_per_model = attempt_limit(config.max_retries) as f64;
    let estimate = models
        .into_iter()
        .filter_map(|model| estimate_request_ceiling(config, request, model))
        .map(|ceiling| ceiling * attempts_per_model)
        .sum::<f64>();

    if let Some(limit) = config.budget.daily_budget_usd {
        if request.daily_spend_usd + estimate > limit {
            return Err(budget_error(format!(
                "本次请求可能使今日成本超过{limit:.6}美元预算"
            )));
        }
    }
    if let Some(limit) = config.budget.monthly_budget_usd {
        if request.monthly_spend_usd + estimate > limit {
            return Err(budget_error(format!(
                "本次请求可能使本月成本超过{limit:.6}美元预算"
            )));
        }
    }
    if let Some(limit) = config.budget.per_request_max_usd {
        if estimate > limit {
            return Err(budget_error(format!(
                "本次请求成本上界{estimate:.6}美元超过单次预算{limit:.6}美元"
            )));
        }
    }
    Ok(())
}

fn estimate_request_ceiling(
    config: &GatewayConfig,
    request: &GatewayRequest,
    model: &str,
) -> Option<f64> {
    let pricing = pricing_for_model(config, model)?;
    let serialized = serde_json::to_string(request).ok()?;
    let estimated_input_tokens = (serialized.chars().count() as f64 / 4.0).ceil();
    let search_cost = config.max_tool_calls as f64 * config.budget.web_search_usd_per_call;
    Some(
        estimated_input_tokens / 1_000_000.0 * pricing.input_usd_per_million
            + config.max_output_tokens as f64 / 1_000_000.0 * pricing.output_usd_per_million
            + search_cost,
    )
}
