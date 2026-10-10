use crate::citations::{parse_citation, parse_source};
use crate::response_fields::parse_usage;
use crate::{
    ApiProtocol, GatewayError, GatewayErrorCategory, PlainTextGatewayResponse,
    StructuredGatewayResponse, WebCitation, WebSource,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub(crate) fn parse_plain_text_success_response(
    protocol: ApiProtocol,
    status: u16,
    provider_request_id: Option<String>,
    value: Value,
) -> Result<PlainTextGatewayResponse, GatewayError> {
    if !(200..300).contains(&status) {
        return Err(parse_provider_error(status, &value));
    }
    let response_id = value
        .get("id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| provider_request_id.clone())
        .unwrap_or_else(|| "provider-response-id-unavailable".to_string());
    let model_id = value
        .get("model")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| "compatible-model".to_string());
    let response_status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("completed")
        .to_string();
    if let Some(refusal) = extract_compatible_refusal(&value) {
        return Err(GatewayError::new(
            GatewayErrorCategory::Refused,
            format!("兼容 API拒绝了本次AI问答请求：{refusal}"),
            false,
            "检查问题内容后重试",
        ));
    }
    let text = extract_compatible_text(protocol, &value).ok_or_else(|| {
        GatewayError::new(
            GatewayErrorCategory::NoResult,
            "兼容 API响应中没有识别到文本内容",
            true,
            "检查所选协议、端点和模型；详细原始响应已写入运行日志",
        )
    })?;
    let usage = parse_usage(value.get("usage"));
    Ok(PlainTextGatewayResponse {
        response_id,
        model_id,
        status: response_status,
        text,
        usage,
        provider_request_id,
        raw_response: value,
    })
}

pub(crate) fn parse_structured_success_response(
    protocol: ApiProtocol,
    status: u16,
    provider_request_id: Option<String>,
    value: Value,
    expected_schema_version: &str,
) -> Result<StructuredGatewayResponse, GatewayError> {
    if !(200..300).contains(&status) {
        return Err(parse_provider_error(status, &value));
    }
    let response_id = value
        .get("id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| provider_request_id.clone())
        .unwrap_or_else(|| "provider-response-id-unavailable".to_string());
    let model_id = value
        .get("model")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| "compatible-model".to_string());
    let response_status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("completed")
        .to_string();
    if let Some(refusal) = extract_compatible_refusal(&value) {
        return Err(GatewayError::new(
            GatewayErrorCategory::Refused,
            format!("兼容 API拒绝了本次API协作请求：{refusal}"),
            false,
            "检查请求内容、附件和预设后重试",
        ));
    }
    let output_text = extract_compatible_text(protocol, &value).ok_or_else(|| {
        GatewayError::new(
            GatewayErrorCategory::NoResult,
            "兼容 API响应中没有识别到文本内容",
            true,
            "检查所选协议、端点和模型；详细原始响应已写入运行日志",
        )
    })?;
    let output = normalize_structured_output(&output_text, expected_schema_version);
    let (citations, sources, search_call_count) = extract_compatible_sources(&value);
    let usage = parse_usage(value.get("usage"));
    Ok(StructuredGatewayResponse {
        response_id,
        model_id,
        status: response_status,
        output,
        citations,
        sources,
        usage,
        search_call_count,
        provider_request_id,
        raw_response: value,
    })
}

fn extract_compatible_text(protocol: ApiProtocol, value: &Value) -> Option<String> {
    let primary = match protocol {
        ApiProtocol::Responses => extract_responses_text(value),
        ApiProtocol::ChatCompletions => extract_chat_completions_text(value),
    };
    primary
        .or_else(|| extract_responses_text(value))
        .or_else(|| extract_chat_completions_text(value))
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}

fn extract_responses_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("output_text").and_then(Value::as_str) {
        if !text.trim().is_empty() {
            return Some(text.to_string());
        }
    }
    let mut parts = Vec::new();
    for item in value
        .get("output")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(text) = item.as_str() {
            parts.push(text.to_string());
            continue;
        }
        for content in item
            .get("content")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(text) = content.as_str() {
                parts.push(text.to_string());
            } else if let Some(text) = content
                .get("text")
                .and_then(Value::as_str)
                .or_else(|| content.get("content").and_then(Value::as_str))
            {
                parts.push(text.to_string());
            }
        }
    }
    (!parts.is_empty()).then(|| parts.join("\n"))
}

fn extract_chat_completions_text(value: &Value) -> Option<String> {
    let choice = value.get("choices").and_then(Value::as_array)?.first()?;
    if let Some(text) = choice.get("text").and_then(Value::as_str) {
        if !text.trim().is_empty() {
            return Some(text.to_string());
        }
    }
    let content = choice.get("message")?.get("content")?;
    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }
    let mut parts = Vec::new();
    for item in content.as_array().into_iter().flatten() {
        if let Some(text) = item.as_str() {
            parts.push(text.to_string());
        } else if let Some(text) = item
            .get("text")
            .and_then(Value::as_str)
            .or_else(|| item.get("content").and_then(Value::as_str))
        {
            parts.push(text.to_string());
        }
    }
    (!parts.is_empty()).then(|| parts.join("\n"))
}

fn extract_compatible_refusal(value: &Value) -> Option<String> {
    value
        .get("refusal")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| {
            value
                .get("choices")
                .and_then(Value::as_array)
                .and_then(|choices| choices.first())
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("refusal"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
}

fn extract_compatible_sources(value: &Value) -> (Vec<WebCitation>, Vec<WebSource>, u32) {
    let mut citations = Vec::new();
    let mut sources = BTreeMap::<String, WebSource>::new();
    let mut search_call_count = 0u32;
    for (output_index, item) in value
        .get("output")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
    {
        if item.get("type").and_then(Value::as_str) == Some("web_search_call") {
            search_call_count = search_call_count.saturating_add(1);
            for source in item
                .get("action")
                .and_then(|action| action.get("sources"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if let Ok(Some(parsed)) = parse_source(source) {
                    sources.entry(parsed.url.clone()).or_insert(parsed);
                }
            }
        }
        for part in item
            .get("content")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            for annotation in part
                .get("annotations")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if annotation.get("type").and_then(Value::as_str) == Some("url_citation") {
                    if let Ok(citation) = parse_citation(annotation, output_index) {
                        sources
                            .entry(citation.url.clone())
                            .or_insert_with(|| WebSource {
                                url: citation.url.clone(),
                                title: Some(citation.title.clone()),
                                domain: citation.domain.clone(),
                            });
                        citations.push(citation);
                    }
                }
            }
        }
    }
    (
        citations,
        sources.into_values().collect(),
        search_call_count,
    )
}

fn normalize_structured_output(text: &str, expected_schema_version: &str) -> Value {
    let cleaned = clean_json_text(text);
    match serde_json::from_str::<Value>(&cleaned) {
        Ok(Value::Object(object)) => normalize_structured_object(object, expected_schema_version),
        Ok(value) => fallback_structured_output(
            serde_json::to_string_pretty(&value).unwrap_or(cleaned),
            expected_schema_version,
            "兼容 API返回了JSON，但顶层不是对象；客户端已按普通文本保留。",
        ),
        Err(_) => fallback_structured_output(
            cleaned,
            expected_schema_version,
            "兼容 API未返回结构化JSON；客户端已保留普通文本，且不会生成文件或数据库写入提案。",
        ),
    }
}

fn normalize_structured_object(
    mut source: Map<String, Value>,
    expected_schema_version: &str,
) -> Value {
    let answer = source
        .remove("answer")
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_else(|| "兼容 API返回了JSON结果。".to_string());
    let summary = source
        .remove("summary")
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_else(|| summarize_text(&answer));
    let mut normalized = Map::new();
    normalized.insert("schema_version".to_string(), json!(expected_schema_version));
    normalized.insert("answer".to_string(), json!(answer));
    normalized.insert("summary".to_string(), json!(summary));
    for key in [
        "key_points",
        "missing_information",
        "warnings",
        "proposed_operations",
        "generated_files",
    ] {
        let value = source
            .remove(key)
            .filter(Value::is_array)
            .unwrap_or_else(|| json!([]));
        normalized.insert(key.to_string(), value);
    }
    Value::Object(normalized)
}

fn fallback_structured_output(
    answer: String,
    expected_schema_version: &str,
    warning: &str,
) -> Value {
    let summary = summarize_text(&answer);
    json!({
        "schema_version": expected_schema_version,
        "answer": answer,
        "summary": summary,
        "key_points": [],
        "missing_information": [],
        "warnings": [warning],
        "proposed_operations": [],
        "generated_files": []
    })
}

fn clean_json_text(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("```") {
        let rest = rest
            .strip_prefix("json")
            .or_else(|| rest.strip_prefix("JSON"))
            .unwrap_or(rest)
            .trim_start();
        return rest.strip_suffix("```").unwrap_or(rest).trim().to_string();
    }
    trimmed.to_string()
}

fn summarize_text(text: &str) -> String {
    text.chars().take(1000).collect::<String>()
}

pub(crate) fn parse_provider_error(status: u16, value: &Value) -> GatewayError {
    let error = value.get("error").unwrap_or(value);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| value.get("detail").and_then(Value::as_str))
        .map(ToString::to_string)
        .unwrap_or_else(|| {
            serde_json::to_string(value)
                .unwrap_or_else(|_| "兼容 API返回了未说明的错误".to_string())
        });
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let error_type = error
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let (category, retryable, action) = match status {
        401 => (
            GatewayErrorCategory::Authentication,
            false,
            "检查Windows凭据管理器中的API密钥是否有效",
        ),
        403 => (
            GatewayErrorCategory::Permission,
            false,
            "确认项目、组织、模型和Web Search工具权限",
        ),
        408 => (
            GatewayErrorCategory::Timeout,
            true,
            "按退避策略重试；持续超时则降低搜索范围或提高超时配置",
        ),
        409 => (
            GatewayErrorCategory::ProviderUnavailable,
            true,
            "按幂等键和退避策略重试",
        ),
        429 => (
            GatewayErrorCategory::RateLimit,
            true,
            "等待限流窗口后重试，并检查并发和账户配额",
        ),
        500..=599 => (
            GatewayErrorCategory::ProviderUnavailable,
            true,
            "OpenAI服务恢复后按幂等策略重试",
        ),
        _ if error_type.contains("model")
            || code.as_deref().is_some_and(|c| c.contains("model")) =>
        {
            (
                GatewayErrorCategory::ModelUnavailable,
                true,
                "切换到已配置的fallback_model，或确认账户可用模型",
            )
        }
        _ => (
            GatewayErrorCategory::Unknown,
            false,
            "查看提供方错误类别，修正请求或配置后重试",
        ),
    };
    GatewayError::new(
        category,
        format!("兼容 API请求失败（HTTP {status}）：{message}"),
        retryable,
        action,
    )
    .with_provider(Some(status), code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_chat_completions_workspace_json_and_usage() {
        let response = json!({
            "id": "chatcmpl_1",
            "model": "gpt-5.5",
            "choices": [{"message": {"content": "```json\n{\"answer\":\"hello\",\"summary\":\"short\"}\n```"}}],
            "usage": {"prompt_tokens": 7, "completion_tokens": 3, "total_tokens": 10}
        });
        let parsed = parse_structured_success_response(
            ApiProtocol::ChatCompletions,
            200,
            Some("req_chat_1".to_string()),
            response,
            "football.api-workspace-response.v2",
        )
        .unwrap();
        assert_eq!(parsed.output["answer"], "hello");
        assert_eq!(
            parsed.output["schema_version"],
            "football.api-workspace-response.v2"
        );
        assert_eq!(parsed.usage.input_tokens, 7);
        assert_eq!(parsed.usage.output_tokens, 3);
    }

    #[test]
    fn preserves_plain_text_workspace_response_without_operations() {
        let response = json!({
            "id": "resp_plain",
            "model": "compatible-model",
            "output_text": "普通上下文回答"
        });
        let parsed = parse_structured_success_response(
            ApiProtocol::Responses,
            200,
            None,
            response,
            "football.api-workspace-response.v2",
        )
        .unwrap();
        assert_eq!(parsed.output["answer"], "普通上下文回答");
        assert_eq!(parsed.output["proposed_operations"], json!([]));
        assert_eq!(parsed.output["generated_files"], json!([]));
    }

    #[test]
    fn maps_rate_limit_to_retryable_error() {
        let error = parse_provider_error(
            429,
            &json!({"error":{"message":"slow down","code":"rate_limit"}}),
        );
        assert_eq!(error.category, GatewayErrorCategory::RateLimit);
        assert!(error.recovery.retryable);
    }
}
