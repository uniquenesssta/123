use async_trait::async_trait;
use chrono::{TimeZone, Utc};
use football_research_gateway::{
    test_openai_connection, ApiKey, ApiKeyProvider, ApiProtocol, ApiWorkspaceWebSearchMode,
    BudgetConfig, CancellationToken, CircuitBreakerConfig, CredentialConfig, CredentialMode,
    GatewayConfig, GatewayError, GatewayErrorCategory, GatewayOperation, GatewayRequest,
    ModelPricing, OpenAiResearchGateway, OpenAiTransport, PlainTextGatewayRequest,
    PlainTextMessage, ReasoningEffort, SearchContextSize, SourcePolicy, StructuredGatewayRequest,
    TokenLimitField, TransportResponse,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
struct TestKeyProvider;

#[async_trait]
impl ApiKeyProvider for TestKeyProvider {
    async fn load(&self, _config: &CredentialConfig) -> Result<ApiKey, GatewayError> {
        ApiKey::new("fixture-credential-value".to_string())
    }
}

#[derive(Default)]
struct FakeTransport {
    responses: Mutex<VecDeque<TransportResponse>>,
    requests: Mutex<Vec<Value>>,
}

impl FakeTransport {
    fn with_responses(responses: Vec<TransportResponse>) -> Self {
        Self {
            responses: Mutex::new(responses.into()),
            requests: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl OpenAiTransport for FakeTransport {
    async fn post_json(
        &self,
        _url: &str,
        _api_key: &ApiKey,
        body: &Value,
        _timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        self.requests.lock().unwrap().push(body.clone());
        self.responses.lock().unwrap().pop_front().ok_or_else(|| {
            GatewayError::new(
                GatewayErrorCategory::Network,
                "no fake response",
                false,
                "fix test",
            )
        })
    }

    async fn get_json(
        &self,
        _url: &str,
        _api_key: &ApiKey,
        _timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Err(GatewayError::new(
            GatewayErrorCategory::Network,
            "unexpected get",
            false,
            "fix test",
        ))
    }

    async fn post_empty(
        &self,
        _url: &str,
        _api_key: &ApiKey,
        _timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Err(GatewayError::new(
            GatewayErrorCategory::Network,
            "unexpected empty post",
            false,
            "fix test",
        ))
    }
}

struct FakeModelTransport {
    response: Mutex<Option<TransportResponse>>,
    requested_urls: Mutex<Vec<String>>,
    request_bodies: Mutex<Vec<Value>>,
}

impl FakeModelTransport {
    fn new(response: TransportResponse) -> Self {
        Self {
            response: Mutex::new(Some(response)),
            requested_urls: Mutex::new(Vec::new()),
            request_bodies: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl OpenAiTransport for FakeModelTransport {
    async fn post_json(
        &self,
        url: &str,
        _api_key: &ApiKey,
        body: &Value,
        _timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        self.requested_urls.lock().unwrap().push(url.to_string());
        self.request_bodies.lock().unwrap().push(body.clone());
        self.response.lock().unwrap().take().ok_or_else(|| {
            GatewayError::new(
                GatewayErrorCategory::Network,
                "no fake response",
                false,
                "fix test",
            )
        })
    }

    async fn get_json(
        &self,
        _url: &str,
        _api_key: &ApiKey,
        _timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Err(GatewayError::new(
            GatewayErrorCategory::Network,
            "unexpected get",
            false,
            "fix test",
        ))
    }

    async fn post_empty(
        &self,
        _url: &str,
        _api_key: &ApiKey,
        _timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Err(GatewayError::new(
            GatewayErrorCategory::Network,
            "unexpected empty post",
            false,
            "fix test",
        ))
    }
}

#[tokio::test]
async fn plain_text_responses_uses_only_model_and_input() {
    let transport = Arc::new(FakeTransport::with_responses(vec![TransportResponse {
        status: 200,
        provider_request_id: Some("req_plain_responses".to_string()),
        body: json!({
            "id": "resp_plain_responses",
            "status": "completed",
            "model": "configured-extraction-model",
            "output": [{
                "type": "message",
                "content": [{"type": "output_text", "text": "普通文本回答", "annotations": []}]
            }],
            "usage": {"input_tokens": 8, "output_tokens": 4, "total_tokens": 12}
        }),
    }]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport.clone(), Arc::new(TestKeyProvider))
            .expect("gateway");
    let request = PlainTextGatewayRequest {
        operation: GatewayOperation::Extraction,
        trace_id: "trace-plain-responses".to_string(),
        static_instructions: "Answer in ordinary text.".to_string(),
        messages: vec![PlainTextMessage {
            role: "user".to_string(),
            content: "你好".to_string(),
        }],
        daily_spend_usd: 0.0,
        monthly_spend_usd: 0.0,
        attempt_number_offset: 0,
    };
    let execution = gateway
        .execute_plain_text(&request, &CancellationToken::new())
        .await
        .expect("plain response");
    assert_eq!(execution.response.text, "普通文本回答");
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["model"], "configured-extraction-model");
    assert!(requests[0].get("input").is_some());
    for field in [
        "messages",
        "tools",
        "tool_choice",
        "text",
        "reasoning",
        "store",
        "metadata",
        "background",
        "max_output_tokens",
        "max_completion_tokens",
        "max_tokens",
    ] {
        assert!(
            requests[0].get(field).is_none(),
            "unexpected field: {field}"
        );
    }
}

#[tokio::test]
async fn plain_text_chat_completions_uses_only_model_and_messages() {
    let transport = Arc::new(FakeTransport::with_responses(vec![TransportResponse {
        status: 200,
        provider_request_id: Some("req_plain_chat".to_string()),
        body: json!({
            "id": "chatcmpl_plain",
            "model": "configured-extraction-model",
            "choices": [{"index": 0, "message": {"role": "assistant", "content": "chat answer"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 7, "completion_tokens": 3, "total_tokens": 10}
        }),
    }]));
    let mut chat_config = config();
    chat_config.api_protocol = ApiProtocol::ChatCompletions;
    chat_config.request_endpoint = Some("https://api.example.com/v1/chat/completions".to_string());
    let gateway =
        OpenAiResearchGateway::new(chat_config, transport.clone(), Arc::new(TestKeyProvider))
            .expect("gateway");
    let request = PlainTextGatewayRequest {
        operation: GatewayOperation::Extraction,
        trace_id: "trace-plain-chat".to_string(),
        static_instructions: "Answer in ordinary text.".to_string(),
        messages: vec![PlainTextMessage {
            role: "user".to_string(),
            content: "hello".to_string(),
        }],
        daily_spend_usd: 0.0,
        monthly_spend_usd: 0.0,
        attempt_number_offset: 0,
    };
    let execution = gateway
        .execute_plain_text(&request, &CancellationToken::new())
        .await
        .expect("plain chat response");
    assert_eq!(execution.response.text, "chat answer");
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["model"], "configured-extraction-model");
    assert!(requests[0].get("messages").is_some());
    for field in [
        "input",
        "tools",
        "tool_choice",
        "text",
        "reasoning",
        "store",
        "metadata",
        "background",
        "max_output_tokens",
        "max_completion_tokens",
        "max_tokens",
    ] {
        assert!(
            requests[0].get(field).is_none(),
            "unexpected field: {field}"
        );
    }
}

#[tokio::test]
async fn structured_workspace_request_omits_background_for_synchronous_compatible_endpoints() {
    let output = json!({
        "schema_version": "workspace.test.v1",
        "answer": "ok",
        "generated_files": []
    });
    let transport = Arc::new(FakeTransport::with_responses(vec![TransportResponse {
        status: 200,
        provider_request_id: Some("req_workspace".to_string()),
        body: json!({
            "id": "resp_workspace",
            "status": "completed",
            "model": "configured-extraction-model",
            "output": [{
                "type": "message",
                "content": [{
                    "type": "output_text",
                    "text": serde_json::to_string(&output).unwrap(),
                    "annotations": []
                }]
            }],
            "usage": {"input_tokens": 10, "output_tokens": 5, "total_tokens": 15}
        }),
    }]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport.clone(), Arc::new(TestKeyProvider))
            .expect("gateway");
    let request = StructuredGatewayRequest {
        operation: GatewayOperation::Extraction,
        trace_id: "trace-workspace".to_string(),
        schema_name: "workspace_test".to_string(),
        schema_version: "workspace.test.v1".to_string(),
        schema: json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "workspace.test.v1",
            "type": "object",
            "additionalProperties": false,
            "required": ["schema_version", "answer", "generated_files"],
            "properties": {
                "schema_version": {"const": "workspace.test.v1"},
                "answer": {"type": "string"},
                "generated_files": {"type": "array", "items": {"type": "object"}}
            }
        }),
        static_instructions: "Return valid structured JSON.".to_string(),
        input: json!({"message": "hello"}),
        enable_web_search: false,
        daily_spend_usd: 0.0,
        monthly_spend_usd: 0.0,
        attempt_number_offset: 0,
    };

    gateway
        .execute_structured(&request, &CancellationToken::new())
        .await
        .expect("structured response");

    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].get("background").is_none());
    assert!(requests[0].get("store").is_none());
    assert!(requests[0].get("metadata").is_none());
    assert!(requests[0].get("reasoning").is_none());
    assert!(requests[0].get("text").is_none());
    assert!(requests[0].get("tools").is_none());
    assert!(requests[0].get("input").is_some());
}

#[tokio::test]
async fn structured_workspace_supports_chat_completions_minimal_payload() {
    let output = json!({
        "schema_version": "workspace.test.v1",
        "answer": "ok",
        "summary": "ok",
        "key_points": [],
        "missing_information": [],
        "warnings": [],
        "proposed_operations": [],
        "generated_files": []
    });
    let transport = Arc::new(FakeTransport::with_responses(vec![TransportResponse {
        status: 200,
        provider_request_id: Some("req_chat_workspace".to_string()),
        body: json!({
            "id": "chatcmpl_workspace",
            "model": "gpt-5.5",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": format!("```json\n{}\n```", serde_json::to_string(&output).unwrap())},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
        }),
    }]));
    let mut chat_config = config();
    chat_config.api_base_url = "https://api.gptsapi.net/v1".to_string();
    chat_config.api_protocol = ApiProtocol::ChatCompletions;
    chat_config.request_endpoint = Some("https://api.gptsapi.net/v1/chat/completions".to_string());
    chat_config.token_limit_field = TokenLimitField::MaxTokens;
    let gateway =
        OpenAiResearchGateway::new(chat_config, transport.clone(), Arc::new(TestKeyProvider))
            .expect("gateway");
    let request = StructuredGatewayRequest {
        operation: GatewayOperation::Extraction,
        trace_id: "trace-chat-workspace".to_string(),
        schema_name: "workspace_test".to_string(),
        schema_version: "workspace.test.v1".to_string(),
        schema: json!({"type":"object"}),
        static_instructions: "Return valid JSON.".to_string(),
        input: json!({"message":"hello"}),
        enable_web_search: false,
        daily_spend_usd: 0.0,
        monthly_spend_usd: 0.0,
        attempt_number_offset: 0,
    };
    let execution = gateway
        .execute_structured(&request, &CancellationToken::new())
        .await
        .expect("chat structured response");
    assert_eq!(execution.response.output["answer"], "ok");
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].get("messages").is_some());
    assert!(requests[0].get("max_tokens").is_none());
    assert!(requests[0].get("max_output_tokens").is_none());
    for field in [
        "input",
        "background",
        "store",
        "metadata",
        "reasoning",
        "text",
        "tools",
        "tool_choice",
    ] {
        assert!(
            requests[0].get(field).is_none(),
            "unexpected field: {field}"
        );
    }
}

#[tokio::test]
async fn structured_workspace_responses_sends_web_search_when_enabled() {
    let output = json!({
        "schema_version": "workspace.test.v1",
        "answer": "verified",
        "summary": "verified",
        "key_points": [],
        "missing_information": [],
        "warnings": [],
        "proposed_operations": [],
        "generated_files": []
    });
    let transport = Arc::new(FakeTransport::with_responses(vec![TransportResponse {
        status: 200,
        provider_request_id: Some("req_workspace_search".to_string()),
        body: json!({
            "id": "resp_workspace_search",
            "status": "completed",
            "model": "configured-research-model",
            "output": [
                {
                    "type": "web_search_call",
                    "action": {"sources": [{"type": "url", "url": "https://example.com/source", "title": "Source"}]}
                },
                {
                    "type": "message",
                    "content": [{
                        "type": "output_text",
                        "text": serde_json::to_string(&output).unwrap(),
                        "annotations": [{
                            "type": "url_citation",
                            "url": "https://example.com/source",
                            "title": "Source",
                            "start_index": 0,
                            "end_index": 8
                        }]
                    }]
                }
            ],
            "usage": {"input_tokens": 20, "output_tokens": 10, "total_tokens": 30}
        }),
    }]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport.clone(), Arc::new(TestKeyProvider))
            .expect("gateway");
    let request = StructuredGatewayRequest {
        operation: GatewayOperation::Research,
        trace_id: "trace-workspace-search".to_string(),
        schema_name: "workspace_test".to_string(),
        schema_version: "workspace.test.v1".to_string(),
        schema: json!({"type":"object"}),
        static_instructions: "Return valid JSON.".to_string(),
        input: json!({"message":"find current information"}),
        enable_web_search: true,
        daily_spend_usd: 0.0,
        monthly_spend_usd: 0.0,
        attempt_number_offset: 0,
    };

    let execution = gateway
        .execute_structured(&request, &CancellationToken::new())
        .await
        .expect("web search structured response");
    assert_eq!(execution.response.search_call_count, 1);
    assert_eq!(execution.response.sources.len(), 1);
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0]["tools"][0]["type"], "web_search");
    assert_eq!(requests[0]["tool_choice"], "auto");
    assert_eq!(requests[0]["include"][0], "web_search_call.action.sources");
    assert!(requests[0].get("max_tokens").is_none());
    assert!(requests[0].get("max_output_tokens").is_none());
}

#[tokio::test]
async fn structured_workspace_auto_mode_retries_without_web_search_when_provider_rejects_tools() {
    let output = json!({
        "schema_version": "workspace.test.v1",
        "answer": "context only",
        "summary": "context only",
        "key_points": [],
        "missing_information": [],
        "warnings": [],
        "proposed_operations": [],
        "generated_files": []
    });
    let transport = Arc::new(FakeTransport::with_responses(vec![
        TransportResponse {
            status: 400,
            provider_request_id: Some("req_tools_rejected".to_string()),
            body: json!({"detail":"Unsupported parameter: tools"}),
        },
        TransportResponse {
            status: 200,
            provider_request_id: Some("req_without_tools".to_string()),
            body: json!({
                "id": "resp_without_tools",
                "status": "completed",
                "model": "configured-research-model",
                "output": [{
                    "type": "message",
                    "content": [{
                        "type": "output_text",
                        "text": serde_json::to_string(&output).unwrap(),
                        "annotations": []
                    }]
                }],
                "usage": {"input_tokens": 20, "output_tokens": 10, "total_tokens": 30}
            }),
        },
    ]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport.clone(), Arc::new(TestKeyProvider))
            .expect("gateway");
    let request = StructuredGatewayRequest {
        operation: GatewayOperation::Research,
        trace_id: "trace-workspace-fallback".to_string(),
        schema_name: "workspace_test".to_string(),
        schema_version: "workspace.test.v1".to_string(),
        schema: json!({"type":"object"}),
        static_instructions: "Return valid JSON.".to_string(),
        input: json!({"message":"find current information"}),
        enable_web_search: true,
        daily_spend_usd: 0.0,
        monthly_spend_usd: 0.0,
        attempt_number_offset: 0,
    };

    let execution = gateway
        .execute_structured(&request, &CancellationToken::new())
        .await
        .expect("automatic no-search fallback");
    assert_eq!(execution.attempts.len(), 2);
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].get("tools").is_some());
    assert!(requests[1].get("tools").is_none());
}

#[tokio::test]
async fn connection_test_posts_to_the_configured_responses_endpoint() {
    let transport = FakeModelTransport::new(TransportResponse {
        status: 200,
        provider_request_id: Some("req_model".to_string()),
        body: json!({
            "id":"resp_connection",
            "object":"response",
            "model":"configured-research-model",
            "status":"completed",
            "output":[]
        }),
    });
    let result = test_openai_connection(
        &config(),
        &TestKeyProvider,
        &transport,
        "configured-research-model",
    )
    .await
    .expect("connection test");
    assert_eq!(result.model_id, "configured-research-model");
    assert_eq!(result.protocol, ApiProtocol::Responses);
    assert_eq!(result.endpoint_url, "https://api.openai.com/v1/responses");
    assert_eq!(result.provider_request_id.as_deref(), Some("req_model"));
    let requested_urls = transport.requested_urls.lock().unwrap();
    assert_eq!(
        requested_urls.as_slice(),
        ["https://api.openai.com/v1/responses"]
    );
    let request_bodies = transport.request_bodies.lock().unwrap();
    assert_eq!(request_bodies[0]["model"], "configured-research-model");
    assert!(request_bodies[0].get("input").is_some());
    assert_eq!(request_bodies[0]["max_output_tokens"], 32);
}

#[tokio::test]
async fn connection_test_supports_chat_completions_compatible_endpoint() {
    let transport = FakeModelTransport::new(TransportResponse {
        status: 200,
        provider_request_id: Some("req_chat".to_string()),
        body: json!({
            "id":"chatcmpl_connection",
            "object":"chat.completion",
            "model":"gpt-5.6-sol",
            "choices":[{
                "index":0,
                "message":{"role":"assistant","content":"OK"},
                "finish_reason":"stop"
            }]
        }),
    });
    let mut chat_config = config();
    chat_config.api_base_url = "https://api.gptsapi.net/v1".to_string();
    chat_config.api_protocol = ApiProtocol::ChatCompletions;
    chat_config.request_endpoint = Some("https://api.gptsapi.net/v1/chat/completions".to_string());
    chat_config.token_limit_field = TokenLimitField::MaxTokens;
    let result = test_openai_connection(&chat_config, &TestKeyProvider, &transport, "gpt-5.6-sol")
        .await
        .expect("chat connection test");
    assert_eq!(result.protocol, ApiProtocol::ChatCompletions);
    assert_eq!(
        result.endpoint_url,
        "https://api.gptsapi.net/v1/chat/completions"
    );
    let request_bodies = transport.request_bodies.lock().unwrap();
    assert!(request_bodies[0].get("messages").is_some());
    assert_eq!(request_bodies[0]["max_tokens"], 32);
    assert!(request_bodies[0].get("max_output_tokens").is_none());
}

fn config() -> GatewayConfig {
    GatewayConfig {
        api_base_url: "https://api.openai.com/v1".to_string(),
        api_protocol: ApiProtocol::Responses,
        request_endpoint: Some("https://api.openai.com/v1/responses".to_string()),
        token_limit_field: TokenLimitField::MaxOutputTokens,
        api_workspace_web_search_mode: ApiWorkspaceWebSearchMode::Auto,
        research_model: "configured-research-model".to_string(),
        extraction_model: "configured-extraction-model".to_string(),
        fallback_model: Some("configured-fallback-model".to_string()),
        reasoning_effort: ReasoningEffort::Medium,
        timeout_seconds: 30,
        max_retries: 1,
        retry_base_delay_ms: 1,
        max_concurrency: 1,
        max_output_tokens: 1000,
        max_tool_calls: 5,
        search_context_size: SearchContextSize::High,
        background_mode: false,
        zero_data_retention_required: false,
        store: false,
        credentials: CredentialConfig {
            mode: CredentialMode::WindowsCredentialManager,
            credential_target: "test".to_string(),
            environment_variable: "OPENAI_API_KEY".to_string(),
            deployment_mode: "local_desktop".to_string(),
        },
        source_policy: SourcePolicy {
            allowed_domains: vec!["example.com".to_string()],
            blocked_domains: vec!["predictions.example".to_string()],
            prohibited_fact_keys: vec!["prediction".to_string(), "odds".to_string()],
            prohibited_content_terms: vec!["betting tip".to_string()],
            https_only: true,
        },
        budget: BudgetConfig {
            daily_budget_usd: Some(10.0),
            monthly_budget_usd: Some(100.0),
            per_request_max_usd: Some(1.0),
            web_search_usd_per_call: 0.01,
            model_pricing: BTreeMap::from([
                (
                    "configured-research-model".to_string(),
                    ModelPricing {
                        input_usd_per_million: 1.0,
                        cached_input_usd_per_million: 0.1,
                        output_usd_per_million: 2.0,
                    },
                ),
                (
                    "configured-extraction-model".to_string(),
                    ModelPricing {
                        input_usd_per_million: 1.0,
                        cached_input_usd_per_million: 0.1,
                        output_usd_per_million: 2.0,
                    },
                ),
                (
                    "configured-fallback-model".to_string(),
                    ModelPricing {
                        input_usd_per_million: 1.0,
                        cached_input_usd_per_million: 0.1,
                        output_usd_per_million: 2.0,
                    },
                ),
            ]),
        },
        circuit_breaker: CircuitBreakerConfig {
            consecutive_failure_threshold: 5,
            open_seconds: 30,
        },
    }
}

fn request() -> GatewayRequest {
    GatewayRequest {
        operation: GatewayOperation::Research,
        trace_id: "trace-1".to_string(),
        match_key: "match-1".to_string(),
        data_cutoff_at: Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 0).unwrap(),
        schema_name: "p4_research".to_string(),
        schema_version: "football.p4-research-output.v2".to_string(),
        schema: json!({
            "$schema":"https://json-schema.org/draft/2020-12/schema",
            "$id":"football.p4-research-output.v2",
            "type":"object",
            "additionalProperties":false,
            "required":["schema_version","match_key","data_cutoff_at","facts","missing_fields"],
            "properties":{}
        }),
        static_instructions: "facts only".to_string(),
        dynamic_context: json!({"home":"Home","away":"Away"}),
        requested_fact_keys: vec!["home_injuries".to_string()],
        daily_spend_usd: 0.0,
        monthly_spend_usd: 0.0,
        attempt_number_offset: 0,
    }
}

fn successful_response() -> TransportResponse {
    let structured = json!({
        "schema_version":"football.p4-research-output.v2",
        "match_key":"match-1",
        "data_cutoff_at":"2026-07-14T10:00:00Z",
        "facts":[{
            "fact_key":"home_injuries.player_a.1",
            "field_key":"home_injuries",
            "subject":{"entity_type":"team","name":"Home","external_id":null},
            "value":{"kind":"string_list","text":null,"number":null,"integer":null,"boolean":null,"strings":["Player A unavailable"]},
            "verification_state":"CONFIRMED",
            "source_urls":["https://example.com/team-news"],
            "published_at":"2026-07-14T08:00:00Z",
            "observed_at":null,
            "effective_at":null,
            "timezone":"UTC"
        }],
        "missing_fields":[]
    });
    TransportResponse {
        status: 200,
        provider_request_id: Some("req_1".to_string()),
        body: json!({
            "id":"resp_1",
            "status":"completed",
            "model":"configured-research-model",
            "usage":{"input_tokens":100,"output_tokens":50,"total_tokens":150,"input_tokens_details":{"cached_tokens":20}},
            "output":[
                {"type":"web_search_call","action":{"sources":[{"type":"url","url":"https://example.com/team-news","title":"Team news"}]}},
                {"type":"message","content":[{"type":"output_text","text":serde_json::to_string(&structured).unwrap(),"annotations":[
                    {"type":"url_citation","url":"https://example.com/team-news","title":"Team news","start_index":0,"end_index":10}
                ]}]}
            ]
        }),
    }
}

#[tokio::test]
async fn builds_responses_web_search_strict_schema_without_secret() {
    let transport = Arc::new(FakeTransport::with_responses(vec![successful_response()]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport.clone(), Arc::new(TestKeyProvider)).unwrap();
    let result = gateway
        .execute(&request(), &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.response.citations.len(), 1);
    assert_eq!(result.response.sources.len(), 1);
    let requests = transport.requests.lock().unwrap();
    let body = &requests[0];
    assert_eq!(body["model"], "configured-research-model");
    assert_eq!(body["tools"][0]["type"], "web_search");
    assert_eq!(
        body["tools"][0]["filters"]["allowed_domains"][0],
        "example.com"
    );
    assert_eq!(body["text"]["format"]["type"], "json_schema");
    assert_eq!(body["text"]["format"]["strict"], true);
    assert!(body["text"]["format"]["schema"].get("$schema").is_none());
    assert!(body["text"]["format"]["schema"].get("$id").is_none());
    assert_eq!(body["store"], false);
    assert_eq!(body["include"][0], "web_search_call.action.sources");
    let serialized = body.to_string();
    assert!(!serialized.contains("fixture-credential-value"));
    assert!(!serialized.to_lowercase().contains("authorization"));
}

#[tokio::test]
async fn extraction_model_still_uses_web_search_and_provider_sources() {
    let mut response = successful_response();
    response.body["model"] = json!("configured-extraction-model");
    let transport = Arc::new(FakeTransport::with_responses(vec![response]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport.clone(), Arc::new(TestKeyProvider)).unwrap();
    let mut value = request();
    value.operation = GatewayOperation::Extraction;
    let result = gateway
        .execute(&value, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.response.model_id, "configured-extraction-model");
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests[0]["model"], "configured-extraction-model");
    assert_eq!(requests[0]["tools"][0]["type"], "web_search");
    assert_eq!(requests[0]["include"][0], "web_search_call.action.sources");
}

#[tokio::test]
async fn retries_rate_limit_then_succeeds_idempotently() {
    let transport = Arc::new(FakeTransport::with_responses(vec![
        TransportResponse {
            status: 429,
            provider_request_id: Some("req_rate".to_string()),
            body: json!({"error":{"message":"rate limited","code":"rate_limit"}}),
        },
        successful_response(),
    ]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport.clone(), Arc::new(TestKeyProvider)).unwrap();
    let result = gateway
        .execute(&request(), &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.attempts.len(), 2);
    assert_eq!(transport.requests.lock().unwrap().len(), 2);
    assert_eq!(
        result.attempts[0].request_fingerprint,
        result.attempts[1].request_fingerprint
    );
}

#[test]
fn rejects_background_mode_when_zero_data_retention_is_required() {
    let mut value = config();
    value.background_mode = true;
    value.zero_data_retention_required = true;
    let error = value.validate().expect_err("must reject");
    assert_eq!(error.category, GatewayErrorCategory::InvalidConfiguration);
}

#[tokio::test]
async fn continues_attempt_numbers_after_persisted_history() {
    let transport = Arc::new(FakeTransport::with_responses(vec![successful_response()]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport, Arc::new(TestKeyProvider)).unwrap();
    let mut value = request();
    value.attempt_number_offset = 7;
    let result = gateway
        .execute(&value, &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.attempts[0].attempt_number, 8);
}

#[tokio::test]
async fn uses_configured_fallback_when_primary_model_is_unavailable() {
    let mut fallback_response = successful_response();
    fallback_response.body["model"] = json!("configured-fallback-model");
    let transport = Arc::new(FakeTransport::with_responses(vec![
        TransportResponse {
            status: 404,
            provider_request_id: Some("req_missing_model".to_string()),
            body: json!({"error":{"message":"model unavailable","code":"model_not_found"}}),
        },
        fallback_response,
    ]));
    let gateway =
        OpenAiResearchGateway::new(config(), transport.clone(), Arc::new(TestKeyProvider)).unwrap();
    let result = gateway
        .execute(&request(), &CancellationToken::new())
        .await
        .unwrap();
    assert_eq!(result.response.model_id, "configured-fallback-model");
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests[0]["model"], "configured-research-model");
    assert_eq!(requests[1]["model"], "configured-fallback-model");
}

#[test]
fn background_mode_requires_store_and_zero_data_retention_disables_both() {
    let mut value = config();
    value.background_mode = true;
    value.store = false;
    assert_eq!(
        value
            .validate()
            .expect_err("background needs store")
            .category,
        GatewayErrorCategory::InvalidConfiguration
    );

    value.store = true;
    value.zero_data_retention_required = true;
    assert_eq!(
        value
            .validate()
            .expect_err("zdr forbids background")
            .category,
        GatewayErrorCategory::InvalidConfiguration
    );
}

#[test]
fn rejects_non_https_remote_endpoint_and_blank_fallback() {
    let mut value = config();
    value.api_base_url = "http://api.openai.example/v1".to_string();
    assert_eq!(
        value
            .validate()
            .expect_err("remote http must fail")
            .category,
        GatewayErrorCategory::InvalidConfiguration
    );

    value.api_base_url = "https://api.openai.com/v1".to_string();
    value.fallback_model = Some("   ".to_string());
    assert_eq!(
        value
            .validate()
            .expect_err("blank fallback must fail")
            .category,
        GatewayErrorCategory::InvalidConfiguration
    );
}
