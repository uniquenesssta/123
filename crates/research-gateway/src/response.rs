use crate::{
    ApiProtocol, CitationLocation, GatewayError, GatewayErrorCategory, GatewayResponse,
    GatewayUsage, PlainTextGatewayResponse, ResearchOutput, StructuredGatewayResponse, WebCitation,
    WebSource,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use url::Url;

pub(crate) fn parse_success_response(
    status: u16,
    provider_request_id: Option<String>,
    value: Value,
) -> Result<GatewayResponse, GatewayError> {
    if !(200..300).contains(&status) {
        return Err(parse_provider_error(status, &value));
    }
    let response_id = required_string(&value, "id")?;
    let model_id = required_string(&value, "model")?;
    let response_status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("completed")
        .to_string();

    let output_items = value
        .get("output")
        .and_then(Value::as_array)
        .ok_or_else(|| schema_error("Responses API响应缺少output数组"))?;
    let mut output_text = None;
    let mut citations = Vec::new();
    let mut sources = BTreeMap::<String, WebSource>::new();
    let mut search_call_count = 0u32;
    let mut refusal = None;

    for (output_index, item) in output_items.iter().enumerate() {
        match item.get("type").and_then(Value::as_str) {
            Some("message") => {
                let content = item
                    .get("content")
                    .and_then(Value::as_array)
                    .ok_or_else(|| schema_error("Responses API message缺少content数组"))?;
                for part in content {
                    match part.get("type").and_then(Value::as_str) {
                        Some("output_text") => {
                            let text = part
                                .get("text")
                                .and_then(Value::as_str)
                                .ok_or_else(|| schema_error("output_text缺少text"))?;
                            if output_text.replace(text.to_string()).is_some() {
                                return Err(schema_error("Responses API返回了多个结构化输出文本"));
                            }
                            if let Some(annotations) =
                                part.get("annotations").and_then(Value::as_array)
                            {
                                for annotation in annotations {
                                    if annotation.get("type").and_then(Value::as_str)
                                        == Some("url_citation")
                                    {
                                        let citation = parse_citation(annotation, output_index)?;
                                        sources.entry(citation.url.clone()).or_insert_with(|| {
                                            WebSource {
                                                url: citation.url.clone(),
                                                title: Some(citation.title.clone()),
                                                domain: citation.domain.clone(),
                                            }
                                        });
                                        citations.push(citation);
                                    }
                                }
                            }
                        }
                        Some("refusal") => {
                            refusal = part
                                .get("refusal")
                                .and_then(Value::as_str)
                                .map(ToString::to_string);
                        }
                        _ => {}
                    }
                }
            }
            Some("web_search_call") => {
                search_call_count = search_call_count.saturating_add(1);
                if let Some(action_sources) = item
                    .get("action")
                    .and_then(|action| action.get("sources"))
                    .and_then(Value::as_array)
                {
                    for source in action_sources {
                        if let Some(parsed) = parse_source(source)? {
                            sources.entry(parsed.url.clone()).or_insert(parsed);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(refusal) = refusal {
        return Err(GatewayError::new(
            GatewayErrorCategory::Refused,
            format!("OpenAI拒绝了本次事实研究请求：{refusal}"),
            false,
            "检查请求是否仅包含公开、可验证的赛事事实，并保留任务为失败状态",
        ));
    }
    let output_text = output_text.ok_or_else(|| {
        GatewayError::new(
            GatewayErrorCategory::NoResult,
            "OpenAI响应没有返回结构化事实结果",
            true,
            "检查搜索来源和请求字段；可在退避后重试或改用fallback_model",
        )
    })?;
    let output: ResearchOutput = serde_json::from_str(&output_text).map_err(|error| {
        GatewayError::new(
            GatewayErrorCategory::SchemaValidation,
            format!("OpenAI结构化输出无法反序列化：{error}"),
            false,
            "保留原始响应，发布修正后的Prompt或Schema版本后重试",
        )
    })?;
    let usage = parse_usage(value.get("usage"));

    Ok(GatewayResponse {
        response_id,
        model_id,
        status: response_status,
        output,
        citations,
        sources: sources.into_values().collect(),
        usage,
        search_call_count,
        provider_request_id,
        raw_response: value,
    })
}

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

fn parse_citation(value: &Value, output_index: usize) -> Result<WebCitation, GatewayError> {
    let url = required_string(value, "url")?;
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(&url)
        .to_string();
    let domain = domain_from_url(&url)?;
    Ok(WebCitation {
        url,
        title,
        domain,
        location: CitationLocation {
            output_index,
            start_index: optional_usize(value, "start_index")?,
            end_index: optional_usize(value, "end_index")?,
        },
    })
}

fn optional_usize(value: &Value, key: &str) -> Result<Option<usize>, GatewayError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| {
            usize::try_from(value).map_err(|_| schema_error(format!("引用字段{key}超出usize范围")))
        })
        .transpose()
}

fn parse_source(value: &Value) -> Result<Option<WebSource>, GatewayError> {
    let Some(url) = value.get("url").and_then(Value::as_str) else {
        return Ok(None);
    };
    let domain = domain_from_url(url)?;
    Ok(Some(WebSource {
        url: url.to_string(),
        title: value
            .get("title")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        domain,
    }))
}

fn parse_usage(value: Option<&Value>) -> GatewayUsage {
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

fn required_string(value: &Value, key: &str) -> Result<String, GatewayError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| schema_error(format!("Responses API响应缺少字符串字段：{key}")))
}

fn domain_from_url(value: &str) -> Result<String, GatewayError> {
    Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(ToString::to_string))
        .map(|host| host.trim_start_matches("www.").to_lowercase())
        .ok_or_else(|| schema_error(format!("引用包含无效URL：{value}")))
}

fn schema_error(message: impl Into<String>) -> GatewayError {
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
    fn parses_structured_output_citations_and_all_sources() {
        let output = json!({
            "schema_version": "football.p4-research-output.v2",
            "match_key": "m1",
            "data_cutoff_at": "2026-07-14T10:00:00Z",
            "facts": [],
            "missing_fields": []
        });
        let response = json!({
            "id": "resp_1",
            "status": "completed",
            "model": "configured-model",
            "usage": {"input_tokens": 10, "output_tokens": 20, "total_tokens": 30,
                "input_tokens_details": {"cached_tokens": 4}},
            "output": [
                {"type": "web_search_call", "action": {"sources": [
                    {"type": "url", "url": "https://example.com/a", "title": "A"},
                    {"type": "url", "url": "https://example.org/b", "title": "B"}
                ]}},
                {"type": "message", "content": [{"type": "output_text",
                    "text": serde_json::to_string(&output).unwrap(),
                    "annotations": [{"type":"url_citation", "url":"https://example.com/a",
                        "title":"A", "start_index":0, "end_index":4}]
                }]}
            ]
        });
        let parsed = parse_success_response(200, Some("req_1".to_string()), response).unwrap();
        assert_eq!(parsed.citations.len(), 1);
        assert_eq!(parsed.sources.len(), 2);
        assert_eq!(parsed.usage.cached_input_tokens, 4);
        assert_eq!(parsed.search_call_count, 1);
    }

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
