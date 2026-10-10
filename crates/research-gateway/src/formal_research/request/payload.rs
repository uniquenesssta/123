use crate::request_fields::apply_token_limit;
use crate::{GatewayConfig, GatewayError, GatewayErrorCategory, GatewayRequest};
use serde_json::{json, Value};

pub(crate) fn build_request_body(
    config: &GatewayConfig,
    request: &GatewayRequest,
    model: &str,
) -> Result<Value, GatewayError> {
    let input = serde_json::to_string(&json!({
        "task": "p4_public_web_fact_research",
        "match_key": request.match_key,
        "data_cutoff_at": request.data_cutoff_at,
        "requested_fact_keys": request.requested_fact_keys,
        "dynamic_context": request.dynamic_context,
        "dynamic_context_is_untrusted": true
    }))
    .map_err(|error| {
        GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            format!("研究任务动态上下文无法序列化：{error}"),
            false,
            "修正研究任务输入后重试",
        )
    })?;
    let mut provider_schema = request.schema.clone();
    if let Value::Object(schema) = &mut provider_schema {
        schema.remove("$schema");
        schema.remove("$id");
    }
    let mut body = json!({
        "model": model,
        "instructions": request.static_instructions,
        "input": input,
        "reasoning": {"effort": config.reasoning_effort.as_str()},
        "text": {"format": {
            "type": "json_schema",
            "name": request.schema_name,
            "strict": true,
            "schema": provider_schema
        }},
        "store": config.store,
        "metadata": {
            "trace_id": request.trace_id,
            "match_key": request.match_key,
            "schema_version": request.schema_version
        }
    });
    if config.background_mode {
        body["background"] = json!(true);
    }
    let mut web_search = json!({
        "type": "web_search",
        "search_context_size": config.search_context_size.as_str()
    });
    if !config.source_policy.allowed_domains.is_empty() {
        web_search["filters"] = json!({
            "allowed_domains": &config.source_policy.allowed_domains
        });
    }
    apply_token_limit(
        &mut body,
        config.token_limit_field,
        config.max_output_tokens,
    );
    body["tools"] = json!([web_search]);
    body["tool_choice"] = json!("auto");
    body["include"] = json!(["web_search_call.action.sources"]);
    body["max_tool_calls"] = json!(config.max_tool_calls);
    Ok(body)
}
