use crate::{GatewayError, GatewayErrorCategory, GatewayRequest};
use serde_json::Value;

pub(crate) fn validate_gateway_request(request: &GatewayRequest) -> Result<(), GatewayError> {
    let valid_schema_name = !request.schema_name.is_empty()
        && request.schema_name.len() <= 64
        && request
            .schema_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-');
    let unique_fact_keys: std::collections::BTreeSet<_> = request
        .requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect();
    if request.trace_id.trim().is_empty()
        || request.trace_id.chars().count() > 64
        || request.match_key.trim().is_empty()
        || request.match_key.chars().count() > 200
        || !valid_schema_name
        || request.schema_version.trim().is_empty()
        || request.static_instructions.trim().is_empty()
        || request.requested_fact_keys.is_empty()
        || request.requested_fact_keys.len() > 31
        || unique_fact_keys.len() != request.requested_fact_keys.len()
        || request
            .requested_fact_keys
            .iter()
            .any(|key| key.trim().is_empty() || key.chars().count() > 100)
    {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "OpenAI研究请求的追踪、比赛、Schema或事实字段契约无效",
            false,
            "修正研究任务输入后重试",
        ));
    }
    let schema = request.schema.as_object().ok_or_else(|| {
        GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "OpenAI严格输出Schema根节点必须是对象",
            false,
            "修正版本化Schema后重试",
        )
    })?;
    if schema.get("type").and_then(Value::as_str) != Some("object")
        || schema.get("additionalProperties").and_then(Value::as_bool) != Some(false)
    {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "OpenAI严格输出Schema必须是type=object且additionalProperties=false",
            false,
            "修正版本化Schema后重试",
        ));
    }
    Ok(())
}
