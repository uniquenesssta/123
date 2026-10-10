use crate::{GatewayError, GatewayErrorCategory, GatewayUsage};
use serde_json::Value;

pub(crate) fn parse_usage(value: Option<&Value>) -> GatewayUsage {
    let value = value.unwrap_or(&Value::Null);
    let input_tokens = value
        .get("input_tokens")
        .or_else(|| value.get("prompt_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let output_tokens = value
        .get("output_tokens")
        .or_else(|| value.get("completion_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    GatewayUsage {
        input_tokens,
        cached_input_tokens: value
            .get("input_tokens_details")
            .or_else(|| value.get("prompt_tokens_details"))
            .and_then(|details| details.get("cached_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0),
        output_tokens,
        total_tokens: value
            .get("total_tokens")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| input_tokens.saturating_add(output_tokens)),
    }
}

pub(crate) fn required_string(value: &Value, key: &str) -> Result<String, GatewayError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| schema_error(format!("Responses API响应缺少字符串字段：{key}")))
}

pub(crate) fn schema_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::SchemaValidation,
        message,
        false,
        "保留原始响应并检查OpenAI响应结构、Prompt和Schema版本",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn usage_keeps_protocol_aliases_precedence_and_saturating_total() {
        let value = json!({"input_tokens":5,"prompt_tokens":99,"output_tokens":3,"completion_tokens":99,
            "input_tokens_details":{"cached_tokens":2},"prompt_tokens_details":{"cached_tokens":99}});
        assert_eq!(
            parse_usage(Some(&value)),
            GatewayUsage {
                input_tokens: 5,
                cached_input_tokens: 2,
                output_tokens: 3,
                total_tokens: 8
            }
        );
        let value = json!({"prompt_tokens":7,"completion_tokens":3,"prompt_tokens_details":{"cached_tokens":1},"total_tokens":12});
        assert_eq!(
            parse_usage(Some(&value)),
            GatewayUsage {
                input_tokens: 7,
                cached_input_tokens: 1,
                output_tokens: 3,
                total_tokens: 12
            }
        );
        let value = json!({"input_tokens":u64::MAX,"output_tokens":1});
        assert_eq!(parse_usage(Some(&value)).total_tokens, u64::MAX);
    }

    #[test]
    fn usage_missing_and_invalid_primary_fields_keep_original_zero_defaults() {
        assert_eq!(parse_usage(None), GatewayUsage::default());
        let value = json!({"input_tokens":"5","prompt_tokens":9,"output_tokens":-1,"completion_tokens":9,
            "input_tokens_details":null,"prompt_tokens_details":{"cached_tokens":9},"total_tokens":"18"});
        assert_eq!(parse_usage(Some(&value)), GatewayUsage::default());
    }
}
