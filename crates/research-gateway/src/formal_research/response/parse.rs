use crate::citations::{parse_citation, parse_source};
use crate::formal_research::schema::decode_output_text;
use crate::response::parse_provider_error;
use crate::response_fields::{parse_usage, required_string, schema_error};
use crate::{GatewayError, GatewayErrorCategory, GatewayResponse, WebSource};
use serde_json::Value;
use std::collections::BTreeMap;

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
    let output = decode_output_text(&output_text)?;
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

    fn response_fixture() -> Value {
        json!({"id":"resp_fixture","model":"model_fixture","output":[{
            "type":"message","content":[{"type":"output_text","text":json!({
                "schema_version":"schema-v1","match_key":"match-1",
                "data_cutoff_at":"2026-07-14T10:00:00Z","facts":[],"missing_fields":[]
            }).to_string()}]
        }]})
    }

    #[test]
    fn formal_envelope_requires_typed_fields_and_never_uses_plain_text_fallback() {
        for (field, message) in [
            ("id", "Responses API响应缺少字符串字段：id"),
            ("model", "Responses API响应缺少字符串字段：model"),
            ("output", "Responses API响应缺少output数组"),
        ] {
            let mut value = response_fixture();
            value[field] = Value::Null;
            let error = parse_success_response(200, None, value).expect_err("invalid envelope");
            assert_eq!(error.category, GatewayErrorCategory::SchemaValidation);
            assert_eq!(error.user_message, message);
            assert!(!error.recovery.retryable);
            assert!(error.provider_status.is_none());
        }
        let mut value = response_fixture();
        value["id"] = json!("");
        value["model"] = json!("");
        let parsed =
            parse_success_response(200, None, value).expect("original empty string policy");
        assert_eq!(parsed.response_id, "");
        assert_eq!(parsed.model_id, "");
        assert_eq!(parsed.status, "completed");
        let error = parse_success_response(
            200,
            None,
            json!({"id":"r","model":"m","output":[],"output_text":"plain"}),
        )
        .expect_err("formal never uses compatible fallback");
        assert_eq!(error.category, GatewayErrorCategory::NoResult);
        assert!(error.recovery.retryable);
    }

    #[test]
    fn formal_message_errors_and_duplicate_text_precede_refusal() {
        let mut value = response_fixture();
        value["output"][0]["content"] = Value::Null;
        assert_eq!(
            parse_success_response(200, None, value)
                .expect_err("content")
                .user_message,
            "Responses API message缺少content数组"
        );
        let mut value = response_fixture();
        value["output"][0]["content"][0]["text"] = json!(7);
        assert_eq!(
            parse_success_response(200, None, value)
                .expect_err("text")
                .user_message,
            "output_text缺少text"
        );
        let mut value = response_fixture();
        let content = value["output"][0]["content"].as_array_mut().unwrap();
        content.push(content[0].clone());
        content.push(json!({"type":"refusal","refusal":"refused"}));
        assert_eq!(
            parse_success_response(200, None, value)
                .expect_err("duplicate before refusal")
                .user_message,
            "Responses API返回了多个结构化输出文本"
        );
    }

    #[test]
    fn refusal_precedes_typed_decode_and_http_errors_precede_envelope() {
        let mut value = response_fixture();
        value["output"][0]["content"][0]["text"] = json!("not json");
        value["output"][0]["content"]
            .as_array_mut()
            .unwrap()
            .push(json!({"type":"refusal","refusal":"facts unavailable"}));
        let error = parse_success_response(200, None, value).expect_err("refusal first");
        assert_eq!(error.category, GatewayErrorCategory::Refused);
        assert_eq!(
            error.user_message,
            "OpenAI拒绝了本次事实研究请求：facts unavailable"
        );
        assert!(!error.recovery.retryable);
        let error = parse_success_response(
            429,
            Some("req_not_promoted".to_string()),
            json!({"error":{"message":"slow","code":"rate_limit"}}),
        )
        .expect_err("provider before envelope");
        assert_eq!(error.category, GatewayErrorCategory::RateLimit);
        assert_eq!(error.provider_status, Some(429));
        assert_eq!(error.provider_code.as_deref(), Some("rate_limit"));
    }

    #[test]
    fn formal_citations_preserve_order_first_source_metadata_usage_and_raw_response() {
        let mut value = response_fixture();
        let message = value["output"][0].clone();
        value["output"] = json!([
            {"type":"web_search_call","action":{"sources":[{"url":"https://www.example.com/z","title":"first"}]}},
            message,
            {"type":"web_search_call","action":{"sources":[{"url":"https://www.example.com/z","title":"second"},{"url":"https://example.com/a"},{"title":"no URL"}]}}
        ]);
        let annotation = json!({"type":"url_citation","url":"https://www.example.com/z","title":"cited","start_index":2,"end_index":5});
        value["output"][1]["content"][0]["annotations"] =
            json!([annotation.clone(),annotation,{"type":"other","url":"bad"}]);
        value["usage"] =
            json!({"input_tokens":8,"output_tokens":2,"input_tokens_details":{"cached_tokens":3}});
        let original = value.clone();
        let parsed = parse_success_response(200, Some("req_original".to_string()), value)
            .expect("valid sources");
        assert_eq!(parsed.raw_response, original);
        assert_eq!(parsed.provider_request_id.as_deref(), Some("req_original"));
        assert_eq!(parsed.citations.len(), 2);
        assert_eq!(parsed.citations[0], parsed.citations[1]);
        assert_eq!(parsed.citations[0].location.output_index, 1);
        assert_eq!(parsed.citations[0].location.start_index, Some(2));
        assert_eq!(parsed.citations[0].domain, "example.com");
        assert_eq!(parsed.sources.len(), 2);
        assert_eq!(parsed.sources[0].url, "https://example.com/a");
        assert_eq!(parsed.sources[0].title, None);
        assert_eq!(parsed.sources[1].title.as_deref(), Some("first"));
        assert_eq!(parsed.search_call_count, 2);
        assert_eq!(parsed.usage.input_tokens, 8);
        assert_eq!(parsed.usage.cached_input_tokens, 3);
        assert_eq!(parsed.usage.total_tokens, 10);
    }

    #[test]
    fn malformed_formal_citation_and_search_source_are_rejected_before_decode() {
        for annotation in [
            json!({"type":"url_citation"}),
            json!({"type":"url_citation","url":"invalid-url"}),
        ] {
            let mut value = response_fixture();
            value["output"][0]["content"][0]["text"] = json!("not json");
            value["output"][0]["content"][0]["annotations"] = json!([annotation]);
            let error =
                parse_success_response(200, None, value).expect_err("citation before decode");
            assert_eq!(error.category, GatewayErrorCategory::SchemaValidation);
            assert!(!error.user_message.contains("无法反序列化"));
        }
        let mut value = response_fixture();
        value["output"]
            .as_array_mut()
            .unwrap()
            .push(json!({"type":"web_search_call","action":{"sources":[{"url":"invalid-url"}]}}));
        assert_eq!(
            parse_success_response(200, None, value)
                .expect_err("search source")
                .user_message,
            "引用包含无效URL：invalid-url"
        );
    }
}
