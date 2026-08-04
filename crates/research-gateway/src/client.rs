use crate::response::{
    parse_plain_text_success_response, parse_provider_error, parse_structured_success_response,
    parse_success_response,
};
use crate::{
    validate_research_output, ApiKey, ApiKeyProvider, ApiProtocol, CancellationToken,
    GatewayAttempt, GatewayConfig, GatewayError, GatewayErrorCategory, GatewayExecution,
    GatewayOperation, GatewayRequest, GatewayResponse, GatewayUsage, ModelPricing,
    OpenAiConnectionTest, PlainTextGatewayExecution, PlainTextGatewayRequest,
    PlainTextGatewayResponse, StructuredGatewayExecution, StructuredGatewayRequest,
    StructuredGatewayResponse, TokenLimitField, ValidationContext,
};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::sync::{Arc, Once};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

#[derive(Debug, Clone)]
pub struct TransportResponse {
    pub status: u16,
    pub provider_request_id: Option<String>,
    pub body: Value,
}

#[async_trait]
pub trait OpenAiTransport: Send + Sync {
    async fn post_json(
        &self,
        url: &str,
        api_key: &ApiKey,
        body: &Value,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError>;

    async fn get_json(
        &self,
        url: &str,
        api_key: &ApiKey,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError>;

    async fn post_empty(
        &self,
        url: &str,
        api_key: &ApiKey,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError>;
}

#[async_trait]
pub trait GatewayAttemptSink: Send + Sync {
    async fn record(&self, attempt: &GatewayAttempt) -> Result<(), GatewayError>;
}

#[derive(Debug, Default)]
struct NoopAttemptSink;

#[async_trait]
impl GatewayAttemptSink for NoopAttemptSink {
    async fn record(&self, _attempt: &GatewayAttempt) -> Result<(), GatewayError> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    pub fn new() -> Result<Self, GatewayError> {
        install_rustls_crypto_provider();
        let client = reqwest::Client::builder()
            .user_agent(concat!(
                "football-match-model-platform/",
                env!("CARGO_PKG_VERSION")
            ))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|error| network_error(format!("无法初始化兼容 API HTTP客户端：{error}")))?;
        Ok(Self { client })
    }

    fn headers(api_key: &ApiKey) -> Result<HeaderMap, GatewayError> {
        let mut headers = HeaderMap::new();
        let authorization = HeaderValue::from_str(&format!("Bearer {}", api_key.expose()))
            .map_err(|_| {
                GatewayError::new(
                    GatewayErrorCategory::MissingCredential,
                    "兼容 API密钥包含无效字符",
                    false,
                    "重新保存Windows凭据管理器中的API密钥",
                )
            })?;
        headers.insert(AUTHORIZATION, authorization);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    async fn execute(
        request: reqwest::RequestBuilder,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        let response = request
            .timeout(timeout)
            .send()
            .await
            .map_err(map_reqwest_error)?;
        let status = response.status().as_u16();
        let provider_request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);
        let bytes = response.bytes().await.map_err(map_reqwest_error)?;
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).map_err(|error| {
                GatewayError::new(
                    GatewayErrorCategory::SchemaValidation,
                    format!("兼容 API返回了无法解析的JSON：{error}"),
                    true,
                    "保留HTTP状态和请求ID后重试；持续发生时检查API端点配置",
                )
                .with_provider(Some(status), None)
            })?
        };
        Ok(TransportResponse {
            status,
            provider_request_id,
            body,
        })
    }
}

#[async_trait]
impl OpenAiTransport for ReqwestTransport {
    async fn post_json(
        &self,
        url: &str,
        api_key: &ApiKey,
        body: &Value,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Self::execute(
            self.client
                .post(url)
                .headers(Self::headers(api_key)?)
                .json(body),
            timeout,
        )
        .await
    }

    async fn get_json(
        &self,
        url: &str,
        api_key: &ApiKey,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Self::execute(
            self.client.get(url).headers(Self::headers(api_key)?),
            timeout,
        )
        .await
    }

    async fn post_empty(
        &self,
        url: &str,
        api_key: &ApiKey,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError> {
        Self::execute(
            self.client.post(url).headers(Self::headers(api_key)?),
            timeout,
        )
        .await
    }
}

pub async fn test_openai_connection(
    config: &GatewayConfig,
    key_provider: &dyn ApiKeyProvider,
    transport: &dyn OpenAiTransport,
    model_id: &str,
) -> Result<OpenAiConnectionTest, GatewayError> {
    config.validate()?;
    let model_id = model_id.trim();
    if model_id.is_empty()
        || model_id.len() > 120
        || !model_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "兼容 API模型ID格式无效",
            false,
            "请输入API服务实际支持的模型ID",
        ));
    }
    let key = key_provider.load(&config.credentials).await?;
    let endpoint = config.request_endpoint();
    let mut body = match config.api_protocol {
        ApiProtocol::Responses => json!({
            "model": model_id,
            "input": [{
                "role": "user",
                "content": [{"type": "input_text", "text": "Reply with OK only."}]
            }]
        }),
        ApiProtocol::ChatCompletions => json!({
            "model": model_id,
            "messages": [{"role": "user", "content": "Reply with OK only."}]
        }),
    };
    apply_token_limit(
        &mut body,
        config.token_limit_field,
        config.max_output_tokens.min(32),
    );
    let started = Instant::now();
    let response = transport
        .post_json(&endpoint, &key, &body, config.timeout())
        .await?;
    if !(200..300).contains(&response.status) {
        return Err(parse_provider_error(response.status, &response.body));
    }
    validate_connection_response(config.api_protocol, &response.body)?;
    let returned_model = response
        .body
        .get("model")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(model_id)
        .to_string();
    Ok(OpenAiConnectionTest {
        model_id: returned_model,
        protocol: config.api_protocol,
        endpoint_url: endpoint,
        provider_request_id: response.provider_request_id,
        latency_ms: elapsed_millis(started),
    })
}

fn validate_connection_response(protocol: ApiProtocol, body: &Value) -> Result<(), GatewayError> {
    let valid = match protocol {
        ApiProtocol::Responses => {
            body.get("id").and_then(Value::as_str).is_some()
                && (body.get("output").and_then(Value::as_array).is_some()
                    || body.get("status").and_then(Value::as_str).is_some())
        }
        ApiProtocol::ChatCompletions => {
            body.get("id").and_then(Value::as_str).is_some()
                && body
                    .get("choices")
                    .and_then(Value::as_array)
                    .is_some_and(|choices| !choices.is_empty())
        }
    };
    if valid {
        Ok(())
    } else {
        Err(GatewayError::new(
            GatewayErrorCategory::SchemaValidation,
            "兼容 API返回结构与所选协议不一致",
            false,
            "检查API Example、请求端点和协议后重试",
        ))
    }
}

fn validate_plain_text_gateway_request(
    request: &PlainTextGatewayRequest,
) -> Result<(), GatewayError> {
    if request.trace_id.trim().is_empty()
        || request.static_instructions.trim().is_empty()
        || request.messages.is_empty()
    {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "AI问答请求缺少追踪ID、系统指令或消息",
            false,
            "修正请求构造后重试",
        ));
    }
    if request.trace_id.chars().count() > 160
        || request.static_instructions.chars().count() > 20_000
        || request.messages.len() > 40
    {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "AI问答请求超过允许长度",
            false,
            "缩短会话历史或系统指令后重试",
        ));
    }
    let mut total_chars = request.static_instructions.chars().count();
    for message in &request.messages {
        if !matches!(message.role.as_str(), "user" | "assistant")
            || message.content.trim().is_empty()
        {
            return Err(GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                "AI问答消息角色或内容无效",
                false,
                "只发送非空的user或assistant消息",
            ));
        }
        total_chars = total_chars.saturating_add(message.content.chars().count());
    }
    if total_chars > 160_000 {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "AI问答上下文超过160000字符上限",
            false,
            "缩短会话历史或关闭当前实体上下文后重试",
        ));
    }
    Ok(())
}

fn validate_structured_gateway_request(
    request: &StructuredGatewayRequest,
) -> Result<(), GatewayError> {
    if request.trace_id.trim().is_empty()
        || request.schema_name.trim().is_empty()
        || request.schema_version.trim().is_empty()
        || request.static_instructions.trim().is_empty()
    {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "API协作请求缺少追踪ID、Schema或指令",
            false,
            "修正请求构造后重试",
        ));
    }
    if request.schema_name.chars().count() > 64
        || request.static_instructions.chars().count() > 30_000
    {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "API协作Schema名称或系统指令超过允许长度",
            false,
            "缩短预设指令后重试",
        ));
    }
    let input_size = serde_json::to_vec(&request.input)
        .map_err(|error| {
            GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                format!("API协作输入无法序列化：{error}"),
                false,
                "修正输入内容后重试",
            )
        })?
        .len();
    if input_size > 8 * 1024 * 1024 {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "API协作输入超过8 MiB上限",
            false,
            "减少附件或拆分对话后重试",
        ));
    }
    Ok(())
}

fn validate_gateway_request(request: &GatewayRequest) -> Result<(), GatewayError> {
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

fn validate_response_id(response_id: &str) -> Result<(), GatewayError> {
    if response_id.is_empty()
        || response_id.len() > 200
        || !response_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
    {
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "OpenAI response_id格式无效",
            false,
            "使用Responses API返回的原始response_id，不要手工拼接URL",
        ));
    }
    Ok(())
}

static RUSTLS_PROVIDER_INSTALL: Once = Once::new();

fn install_rustls_crypto_provider() {
    RUSTLS_PROVIDER_INSTALL.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

struct CircuitState {
    consecutive_failures: u32,
    open_until: Option<Instant>,
}

pub struct OpenAiResearchGateway {
    config: GatewayConfig,
    transport: Arc<dyn OpenAiTransport>,
    key_provider: Arc<dyn ApiKeyProvider>,
    concurrency: Arc<Semaphore>,
    circuit: Mutex<CircuitState>,
}

impl OpenAiResearchGateway {
    pub fn new(
        config: GatewayConfig,
        transport: Arc<dyn OpenAiTransport>,
        key_provider: Arc<dyn ApiKeyProvider>,
    ) -> Result<Self, GatewayError> {
        config.validate()?;
        Ok(Self {
            concurrency: Arc::new(Semaphore::new(config.max_concurrency)),
            config,
            transport,
            key_provider,
            circuit: Mutex::new(CircuitState {
                consecutive_failures: 0,
                open_until: None,
            }),
        })
    }

    pub fn config(&self) -> &GatewayConfig {
        &self.config
    }

    pub async fn execute(
        &self,
        request: &GatewayRequest,
        cancellation: &CancellationToken,
    ) -> Result<GatewayExecution, GatewayError> {
        self.execute_with_sink(request, cancellation, &NoopAttemptSink)
            .await
    }

    pub async fn execute_plain_text(
        &self,
        request: &PlainTextGatewayRequest,
        cancellation: &CancellationToken,
    ) -> Result<PlainTextGatewayExecution, GatewayError> {
        self.execute_plain_text_with_sink(request, cancellation, &NoopAttemptSink)
            .await
    }

    pub async fn execute_plain_text_with_sink(
        &self,
        request: &PlainTextGatewayRequest,
        cancellation: &CancellationToken,
        attempt_sink: &dyn GatewayAttemptSink,
    ) -> Result<PlainTextGatewayExecution, GatewayError> {
        validate_plain_text_gateway_request(request)?;
        if self.config.background_mode {
            return Err(GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                "AI问答不支持后台请求模式",
                false,
                "关闭background后重试；AI问答使用同步Responses或Chat Completions请求",
            ));
        }
        self.check_circuit().await?;
        self.check_plain_text_budget(request)?;
        let permit = tokio::select! {
            result = self.concurrency.clone().acquire_owned() => result.map_err(|_| {
                GatewayError::new(
                    GatewayErrorCategory::ConcurrencyLimit,
                    "AI问答并发控制器已关闭",
                    true,
                    "重新启动应用后重试",
                )
            })?,
            _ = cancellation.cancelled() => return Err(cancelled_error()),
        };
        let _permit = permit;
        let key = self.key_provider.load(&self.config.credentials).await?;
        let mut models = vec![self.model_for_operation(request.operation).to_string()];
        if let Some(fallback) = self.config.fallback_model.as_ref() {
            if !models.contains(fallback) {
                models.push(fallback.clone());
            }
        }
        let mut attempts = Vec::new();
        let mut last_error = None;
        let mut attempt_sequence = request.attempt_number_offset;
        let total_attempt_limit = self.config.max_retries.saturating_add(1);

        'model_loop: for (model_index, model) in models.iter().enumerate() {
            for retry_index in 1..=total_attempt_limit {
                attempt_sequence = attempt_sequence.saturating_add(1);
                if cancellation.is_cancelled() {
                    return Err(cancelled_error());
                }
                let body = self.build_plain_text_request_body(request, model)?;
                let request_fingerprint = fingerprint(&body)?;
                let started_at = Utc::now();
                let started = Instant::now();
                let request_endpoint = self.config.request_endpoint();
                let transport_result = tokio::select! {
                    result = self.transport.post_json(
                        &request_endpoint,
                        &key,
                        &body,
                        self.config.timeout(),
                    ) => result,
                    _ = cancellation.cancelled() => return Err(cancelled_error()),
                };
                let finished_at = Utc::now();
                match transport_result {
                    Ok(transport) => {
                        let provider_status = transport.status;
                        let provider_request_id = transport.provider_request_id.clone();
                        let raw_body = transport.body.clone();
                        match parse_plain_text_success_response(
                            self.config.api_protocol,
                            transport.status,
                            transport.provider_request_id,
                            transport.body,
                        ) {
                            Ok(response) => {
                                let estimated_cost =
                                    self.estimate_plain_text_actual_cost(&response);
                                let attempt = GatewayAttempt {
                                    attempt_number: attempt_sequence,
                                    model_id: model.clone(),
                                    request_fingerprint,
                                    request_payload: body,
                                    response_id: Some(response.response_id.clone()),
                                    provider_request_id: response.provider_request_id.clone(),
                                    provider_status: Some(provider_status),
                                    status: response.status.clone(),
                                    usage: response.usage.clone(),
                                    latency_ms: elapsed_millis(started),
                                    search_call_count: 0,
                                    estimated_cost_usd: estimated_cost,
                                    raw_response: Some(response.raw_response.clone()),
                                    error: None,
                                    started_at,
                                    finished_at,
                                };
                                attempt_sink.record(&attempt).await?;
                                attempts.push(attempt);
                                self.record_success().await;
                                return Ok(PlainTextGatewayExecution { response, attempts });
                            }
                            Err(error) => {
                                let attempt = GatewayAttempt {
                                    attempt_number: attempt_sequence,
                                    model_id: model.clone(),
                                    request_fingerprint,
                                    request_payload: body,
                                    response_id: response_id_from_body(&raw_body),
                                    provider_request_id,
                                    provider_status: Some(provider_status),
                                    status: raw_body
                                        .get("status")
                                        .and_then(Value::as_str)
                                        .unwrap_or("failed")
                                        .to_string(),
                                    usage: GatewayUsage::default(),
                                    latency_ms: elapsed_millis(started),
                                    search_call_count: 0,
                                    estimated_cost_usd: None,
                                    raw_response: Some(raw_body),
                                    error: Some(error.clone()),
                                    started_at,
                                    finished_at,
                                };
                                attempt_sink.record(&attempt).await?;
                                attempts.push(attempt);
                                self.record_failure(&error).await;
                                let has_fallback = model_index + 1 < models.len();
                                let is_last_retry = retry_index >= total_attempt_limit;
                                last_error = Some(error.clone());
                                if error.category == GatewayErrorCategory::ModelUnavailable
                                    && has_fallback
                                {
                                    continue 'model_loop;
                                }
                                if !error.recovery.retryable {
                                    return Err(error);
                                }
                                if is_last_retry {
                                    if has_fallback {
                                        continue 'model_loop;
                                    }
                                    return Err(error);
                                }
                            }
                        }
                    }
                    Err(error) => {
                        let attempt = GatewayAttempt {
                            attempt_number: attempt_sequence,
                            model_id: model.clone(),
                            request_fingerprint,
                            request_payload: body,
                            response_id: None,
                            provider_request_id: None,
                            provider_status: error.provider_status,
                            status: "failed".to_string(),
                            usage: GatewayUsage::default(),
                            latency_ms: elapsed_millis(started),
                            search_call_count: 0,
                            estimated_cost_usd: None,
                            raw_response: None,
                            error: Some(error.clone()),
                            started_at,
                            finished_at,
                        };
                        attempt_sink.record(&attempt).await?;
                        attempts.push(attempt);
                        self.record_failure(&error).await;
                        let has_fallback = model_index + 1 < models.len();
                        let is_last_retry = retry_index >= total_attempt_limit;
                        last_error = Some(error.clone());
                        if !error.recovery.retryable {
                            return Err(error);
                        }
                        if is_last_retry {
                            if has_fallback {
                                continue 'model_loop;
                            }
                            return Err(error);
                        }
                    }
                }
                let delay = retry_delay(self.config.retry_base_delay_ms, retry_index);
                tokio::select! {
                    _ = tokio::time::sleep(delay) => {},
                    _ = cancellation.cancelled() => return Err(cancelled_error()),
                }
            }
        }
        Err(last_error.unwrap_or_else(|| {
            GatewayError::new(
                GatewayErrorCategory::Unknown,
                "AI问答请求未返回结果",
                true,
                "检查兼容 API配置后重试",
            )
        }))
    }

    pub async fn execute_structured(
        &self,
        request: &StructuredGatewayRequest,
        cancellation: &CancellationToken,
    ) -> Result<StructuredGatewayExecution, GatewayError> {
        self.execute_structured_with_sink(request, cancellation, &NoopAttemptSink)
            .await
    }

    pub async fn execute_structured_with_sink(
        &self,
        request: &StructuredGatewayRequest,
        cancellation: &CancellationToken,
        attempt_sink: &dyn GatewayAttemptSink,
    ) -> Result<StructuredGatewayExecution, GatewayError> {
        validate_structured_gateway_request(request)?;
        if self.config.background_mode {
            return Err(GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                "API协作工作台不支持后台请求模式",
                false,
                "关闭background后重试；API协作使用同步Responses或Chat Completions请求",
            ));
        }
        self.check_circuit().await?;
        self.check_structured_budget(request)?;
        let permit = tokio::select! {
            result = self.concurrency.clone().acquire_owned() => result.map_err(|_| {
                GatewayError::new(
                    GatewayErrorCategory::ConcurrencyLimit,
                    "API协作并发控制器已关闭",
                    true,
                    "重新启动应用后重试",
                )
            })?,
            _ = cancellation.cancelled() => return Err(cancelled_error()),
        };
        let _permit = permit;
        let key = self.key_provider.load(&self.config.credentials).await?;
        let mut models = vec![self.model_for_operation(request.operation).to_string()];
        if let Some(fallback) = self.config.fallback_model.as_ref() {
            if !models.contains(fallback) {
                models.push(fallback.clone());
            }
        }
        let mut attempts = Vec::new();
        let mut last_error = None;
        let mut attempt_sequence = request.attempt_number_offset;
        let mut structured_web_search_enabled = request.enable_web_search
            && self.config.api_protocol == ApiProtocol::Responses
            && self
                .config
                .api_workspace_web_search_mode
                .allows_responses_web_search();
        let automatic_web_search_fallback = structured_web_search_enabled
            && self
                .config
                .api_workspace_web_search_mode
                .allows_automatic_fallback();
        let total_attempt_limit = self
            .config
            .max_retries
            .saturating_add(1)
            .saturating_add(u32::from(automatic_web_search_fallback));

        'model_loop: for (model_index, model) in models.iter().enumerate() {
            'retry_loop: for retry_index in 1..=total_attempt_limit {
                attempt_sequence = attempt_sequence.saturating_add(1);
                if cancellation.is_cancelled() {
                    return Err(cancelled_error());
                }
                let body = self.build_structured_request_body(
                    request,
                    model,
                    structured_web_search_enabled,
                )?;
                let request_fingerprint = fingerprint(&body)?;
                let started_at = Utc::now();
                let started = Instant::now();
                let request_endpoint = self.config.request_endpoint();
                let transport_result = tokio::select! {
                    result = self.transport.post_json(
                        &request_endpoint,
                        &key,
                        &body,
                        self.config.timeout(),
                    ) => result,
                    _ = cancellation.cancelled() => return Err(cancelled_error()),
                };
                let finished_at = Utc::now();
                match transport_result {
                    Ok(transport) => {
                        let provider_status = transport.status;
                        let provider_request_id = transport.provider_request_id.clone();
                        let raw_body = transport.body.clone();
                        match parse_structured_success_response(
                            self.config.api_protocol,
                            transport.status,
                            transport.provider_request_id,
                            transport.body,
                            &request.schema_version,
                        ) {
                            Ok(response) => {
                                let estimated_cost =
                                    self.estimate_structured_actual_cost(&response);
                                let attempt = GatewayAttempt {
                                    attempt_number: attempt_sequence,
                                    model_id: model.clone(),
                                    request_fingerprint,
                                    request_payload: body,
                                    response_id: Some(response.response_id.clone()),
                                    provider_request_id: response.provider_request_id.clone(),
                                    provider_status: Some(provider_status),
                                    status: response.status.clone(),
                                    usage: response.usage.clone(),
                                    latency_ms: elapsed_millis(started),
                                    search_call_count: response.search_call_count,
                                    estimated_cost_usd: estimated_cost,
                                    raw_response: Some(response.raw_response.clone()),
                                    error: None,
                                    started_at,
                                    finished_at,
                                };
                                attempt_sink.record(&attempt).await?;
                                attempts.push(attempt);
                                self.record_success().await;
                                return Ok(StructuredGatewayExecution { response, attempts });
                            }
                            Err(error) => {
                                let response_id = response_id_from_body(&raw_body);
                                let should_fallback_without_web_search =
                                    structured_web_search_enabled
                                        && automatic_web_search_fallback
                                        && unsupported_structured_web_search(
                                            provider_status,
                                            &raw_body,
                                        );
                                let attempt = GatewayAttempt {
                                    attempt_number: attempt_sequence,
                                    model_id: model.clone(),
                                    request_fingerprint,
                                    request_payload: body,
                                    response_id,
                                    provider_request_id,
                                    provider_status: Some(provider_status),
                                    status: raw_body
                                        .get("status")
                                        .and_then(Value::as_str)
                                        .unwrap_or("failed")
                                        .to_string(),
                                    usage: GatewayUsage::default(),
                                    latency_ms: elapsed_millis(started),
                                    search_call_count: 0,
                                    estimated_cost_usd: None,
                                    raw_response: Some(raw_body.clone()),
                                    error: Some(error.clone()),
                                    started_at,
                                    finished_at,
                                };
                                attempt_sink.record(&attempt).await?;
                                attempts.push(attempt);
                                if should_fallback_without_web_search {
                                    structured_web_search_enabled = false;
                                    last_error = Some(error);
                                    continue 'retry_loop;
                                }
                                self.record_failure(&error).await;
                                let retryable = error.recovery.retryable;
                                let is_last_retry = retry_index >= total_attempt_limit;
                                let has_fallback = model_index + 1 < models.len();
                                last_error = Some(error.clone());
                                if error.category == GatewayErrorCategory::ModelUnavailable
                                    && has_fallback
                                {
                                    continue 'model_loop;
                                }
                                if !retryable {
                                    return Err(error);
                                }
                                if is_last_retry {
                                    if has_fallback {
                                        continue 'model_loop;
                                    }
                                    return Err(error);
                                }
                            }
                        }
                    }
                    Err(error) => {
                        let attempt = GatewayAttempt {
                            attempt_number: attempt_sequence,
                            model_id: model.clone(),
                            request_fingerprint,
                            request_payload: body,
                            response_id: None,
                            provider_request_id: None,
                            provider_status: error.provider_status,
                            status: "failed".to_string(),
                            usage: GatewayUsage::default(),
                            latency_ms: elapsed_millis(started),
                            search_call_count: 0,
                            estimated_cost_usd: None,
                            raw_response: None,
                            error: Some(error.clone()),
                            started_at,
                            finished_at,
                        };
                        attempt_sink.record(&attempt).await?;
                        attempts.push(attempt);
                        self.record_failure(&error).await;
                        let is_last_retry = retry_index >= total_attempt_limit;
                        let has_fallback = model_index + 1 < models.len();
                        last_error = Some(error.clone());
                        if !error.recovery.retryable {
                            return Err(error);
                        }
                        if is_last_retry {
                            if has_fallback {
                                continue 'model_loop;
                            }
                            return Err(error);
                        }
                    }
                }
                let delay = retry_delay(self.config.retry_base_delay_ms, retry_index);
                tokio::select! {
                    _ = tokio::time::sleep(delay) => {},
                    _ = cancellation.cancelled() => return Err(cancelled_error()),
                }
            }
        }
        Err(last_error.unwrap_or_else(|| {
            GatewayError::new(
                GatewayErrorCategory::Unknown,
                "API协作请求未返回结果",
                true,
                "检查兼容 API配置后重试",
            )
        }))
    }

    pub async fn execute_with_sink(
        &self,
        request: &GatewayRequest,
        cancellation: &CancellationToken,
        attempt_sink: &dyn GatewayAttemptSink,
    ) -> Result<GatewayExecution, GatewayError> {
        validate_gateway_request(request)?;
        if self.config.api_protocol != ApiProtocol::Responses {
            return Err(GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                "P4正式联网研究仅支持Responses协议",
                false,
                "在兼容 API设置中粘贴并选择/v1/responses示例；Chat Completions仅用于连通性测试",
            ));
        }
        self.check_circuit().await?;
        self.check_budget(request)?;
        let permit = tokio::select! {
            result = self.concurrency.clone().acquire_owned() => result.map_err(|_| {
                GatewayError::new(
                    GatewayErrorCategory::ConcurrencyLimit,
                    "OpenAI研究网关并发控制器已关闭",
                    true,
                    "重新启动应用后重试",
                )
            })?,
            _ = cancellation.cancelled() => return Err(cancelled_error()),
        };
        let _permit = permit;
        let key = self.key_provider.load(&self.config.credentials).await?;
        let mut models = vec![self.model_for_operation(request.operation).to_string()];
        if let Some(fallback) = self.config.fallback_model.as_ref() {
            if !models.contains(fallback) {
                models.push(fallback.clone());
            }
        }
        let mut attempts = Vec::new();
        let mut last_error = None;
        let mut attempt_sequence = request.attempt_number_offset;

        'model_loop: for (model_index, model) in models.iter().enumerate() {
            for retry_index in 1..=self.config.max_retries.saturating_add(1) {
                attempt_sequence = attempt_sequence.saturating_add(1);
                let attempt_number = attempt_sequence;
                if cancellation.is_cancelled() {
                    return Err(cancelled_error());
                }
                let body = self.build_request_body(request, model)?;
                let request_fingerprint = fingerprint(&body)?;
                let started_at = Utc::now();
                let started = Instant::now();
                let request_endpoint = self.config.request_endpoint();
                let transport_result = tokio::select! {
                    result = self.transport.post_json(
                        &request_endpoint,
                        &key,
                        &body,
                        self.config.timeout(),
                    ) => result,
                    _ = cancellation.cancelled() => return Err(cancelled_error()),
                };

                let (result, transport_metadata) = match transport_result {
                    Ok(response) => {
                        let metadata = Some((
                            response.status,
                            response.provider_request_id.clone(),
                            response.body.clone(),
                        ));
                        (
                            self.finish_provider_response(request, response, &key, cancellation)
                                .await,
                            metadata,
                        )
                    }
                    Err(error) => (Err(error), None),
                };
                let finished_at = Utc::now();
                match result {
                    Ok(response) => {
                        let estimated_cost = self.estimate_actual_cost(&response);
                        let attempt = GatewayAttempt {
                            attempt_number,
                            model_id: model.clone(),
                            request_fingerprint,
                            request_payload: body.clone(),
                            response_id: Some(response.response_id.clone()),
                            provider_request_id: response.provider_request_id.clone(),
                            provider_status: transport_metadata.as_ref().map(|value| value.0),
                            status: response.status.clone(),
                            usage: response.usage.clone(),
                            latency_ms: elapsed_millis(started),
                            search_call_count: response.search_call_count,
                            estimated_cost_usd: estimated_cost,
                            raw_response: Some(response.raw_response.clone()),
                            error: None,
                            started_at,
                            finished_at,
                        };
                        attempt_sink.record(&attempt).await?;
                        attempts.push(attempt);
                        self.record_success().await;
                        return Ok(GatewayExecution { response, attempts });
                    }
                    Err(error) => {
                        let retryable = error.recovery.retryable;
                        let is_last_retry = retry_index > self.config.max_retries;
                        let has_fallback = model_index + 1 < models.len();
                        let response_id = transport_metadata
                            .as_ref()
                            .and_then(|value| response_id_from_body(&value.2));
                        let attempt = GatewayAttempt {
                            attempt_number,
                            model_id: model.clone(),
                            request_fingerprint,
                            request_payload: body.clone(),
                            response_id,
                            provider_request_id: transport_metadata
                                .as_ref()
                                .and_then(|value| value.1.clone()),
                            provider_status: error
                                .provider_status
                                .or_else(|| transport_metadata.as_ref().map(|value| value.0)),
                            status: transport_metadata
                                .as_ref()
                                .and_then(|value| value.2.get("status"))
                                .and_then(Value::as_str)
                                .filter(|status| {
                                    matches!(
                                        *status,
                                        "queued"
                                            | "in_progress"
                                            | "failed"
                                            | "cancelled"
                                            | "incomplete"
                                    )
                                })
                                .unwrap_or("failed")
                                .to_string(),
                            usage: GatewayUsage::default(),
                            latency_ms: elapsed_millis(started),
                            search_call_count: 0,
                            estimated_cost_usd: None,
                            raw_response: transport_metadata.as_ref().map(|value| value.2.clone()),
                            error: Some(error.clone()),
                            started_at,
                            finished_at,
                        };
                        attempt_sink.record(&attempt).await?;
                        attempts.push(attempt);
                        self.record_failure(&error).await;
                        last_error = Some(error.clone());

                        if error.category == GatewayErrorCategory::ModelUnavailable && has_fallback
                        {
                            continue 'model_loop;
                        }
                        if !retryable {
                            return Err(error);
                        }
                        if is_last_retry {
                            if has_fallback
                                && matches!(
                                    error.category,
                                    GatewayErrorCategory::Network
                                        | GatewayErrorCategory::Timeout
                                        | GatewayErrorCategory::RateLimit
                                        | GatewayErrorCategory::ProviderUnavailable
                                )
                            {
                                continue 'model_loop;
                            }
                            return Err(error);
                        }

                        let delay = retry_delay(self.config.retry_base_delay_ms, retry_index);
                        tokio::select! {
                            _ = tokio::time::sleep(delay) => {},
                            _ = cancellation.cancelled() => return Err(cancelled_error()),
                        }
                    }
                }
            }
        }
        Err(last_error.unwrap_or_else(|| {
            GatewayError::new(
                GatewayErrorCategory::Unknown,
                "OpenAI研究任务没有产生可用结果",
                false,
                "检查模型、来源策略和失败日志后重试",
            )
        }))
    }

    pub async fn resume(
        &self,
        request: &GatewayRequest,
        response_id: &str,
        cancellation: &CancellationToken,
    ) -> Result<GatewayResponse, GatewayError> {
        validate_gateway_request(request)?;
        validate_response_id(response_id)?;
        if !self.config.background_mode || !self.config.store {
            return Err(GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                "后台任务恢复需要background=true且store=true",
                false,
                "启用受控后台模式后再恢复response_id；零数据保留模式不能使用后台恢复",
            ));
        }
        let key = self.key_provider.load(&self.config.credentials).await?;
        let response = self
            .poll_background(response_id, &key, cancellation)
            .await?;
        self.parse_and_validate(request, response)
    }

    pub async fn cancel_remote(&self, response_id: &str) -> Result<(), GatewayError> {
        validate_response_id(response_id)?;
        let key = self.key_provider.load(&self.config.credentials).await?;
        let endpoint = format!(
            "{}/responses/{response_id}/cancel",
            self.config.api_base_url.trim_end_matches('/')
        );
        let response = self
            .transport
            .post_empty(&endpoint, &key, self.config.timeout())
            .await?;
        if !(200..300).contains(&response.status) {
            return Err(parse_provider_error(response.status, &response.body));
        }
        Ok(())
    }

    async fn finish_provider_response(
        &self,
        request: &GatewayRequest,
        response: TransportResponse,
        key: &ApiKey,
        cancellation: &CancellationToken,
    ) -> Result<GatewayResponse, GatewayError> {
        if !(200..300).contains(&response.status) {
            return Err(parse_provider_error(response.status, &response.body));
        }
        let status = response
            .body
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("completed")
            .to_string();
        match status.as_str() {
            "queued" | "in_progress" if self.config.background_mode => {
                let response_id =
                    response
                        .body
                        .get("id")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            GatewayError::new(
                                GatewayErrorCategory::SchemaValidation,
                                "后台OpenAI响应缺少response_id",
                                false,
                                "保留原始响应并检查Responses API版本",
                            )
                        })?;
                let completed = self.poll_background(response_id, key, cancellation).await?;
                self.parse_and_validate(request, completed)
            }
            "queued" | "in_progress" => Err(GatewayError::new(
                GatewayErrorCategory::SchemaValidation,
                "同步OpenAI请求返回了后台任务状态",
                true,
                "检查background_mode和store配置后按幂等键重试",
            )),
            "failed" | "incomplete" => Err(GatewayError::new(
                GatewayErrorCategory::ProviderUnavailable,
                "OpenAI研究任务未完成",
                true,
                "保留response_id和原始响应，检查incomplete_details后重试",
            )),
            "cancelled" => Err(cancelled_error()),
            "completed" => self.parse_and_validate(request, response),
            other => Err(GatewayError::new(
                GatewayErrorCategory::SchemaValidation,
                format!("未知OpenAI响应状态：{other}"),
                false,
                "检查Responses API响应版本",
            )),
        }
    }

    async fn poll_background(
        &self,
        response_id: &str,
        key: &ApiKey,
        cancellation: &CancellationToken,
    ) -> Result<TransportResponse, GatewayError> {
        let endpoint = format!(
            "{}/responses/{response_id}",
            self.config.api_base_url.trim_end_matches('/')
        );
        let deadline = Instant::now() + self.config.timeout();
        loop {
            if cancellation.is_cancelled() {
                let cancel_endpoint = format!("{endpoint}/cancel");
                let _ = self
                    .transport
                    .post_empty(&cancel_endpoint, key, Duration::from_secs(10))
                    .await;
                return Err(cancelled_error());
            }
            if Instant::now() >= deadline {
                return Err(GatewayError::new(
                    GatewayErrorCategory::Timeout,
                    "OpenAI后台研究任务等待超时",
                    true,
                    "保留response_id，稍后从客户端恢复轮询或取消远端任务",
                ));
            }
            let response = tokio::select! {
                result = self.transport.get_json(&endpoint, key, self.config.timeout()) => result?,
                _ = cancellation.cancelled() => continue,
            };
            if !(200..300).contains(&response.status) {
                return Err(parse_provider_error(response.status, &response.body));
            }
            match response
                .body
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("completed")
            {
                "queued" | "in_progress" => {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(1)) => {},
                        _ = cancellation.cancelled() => {},
                    }
                }
                "completed" => return Ok(response),
                "cancelled" => return Err(cancelled_error()),
                "failed" | "incomplete" => {
                    return Err(GatewayError::new(
                        GatewayErrorCategory::ProviderUnavailable,
                        "OpenAI后台研究任务未完成",
                        true,
                        "查看response_id对应的原始状态和错误后按幂等键重试",
                    ));
                }
                other => {
                    return Err(GatewayError::new(
                        GatewayErrorCategory::SchemaValidation,
                        format!("未知OpenAI后台状态：{other}"),
                        false,
                        "检查Responses API响应版本",
                    ));
                }
            }
        }
    }

    fn parse_and_validate(
        &self,
        request: &GatewayRequest,
        response: TransportResponse,
    ) -> Result<GatewayResponse, GatewayError> {
        let parsed =
            parse_success_response(response.status, response.provider_request_id, response.body)?;
        validate_research_output(
            &parsed.output,
            &ValidationContext {
                match_key: &request.match_key,
                schema_version: &request.schema_version,
                data_cutoff_at: request.data_cutoff_at,
                requested_fact_keys: &request.requested_fact_keys,
                source_policy: &self.config.source_policy,
                citations: &parsed.citations,
                sources: &parsed.sources,
            },
        )?;
        Ok(parsed)
    }

    fn build_request_body(
        &self,
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
            "reasoning": {"effort": self.config.reasoning_effort.as_str()},
            "text": {"format": {
                "type": "json_schema",
                "name": request.schema_name,
                "strict": true,
                "schema": provider_schema
            }},
            "store": self.config.store,
            "metadata": {
                "trace_id": request.trace_id,
                "match_key": request.match_key,
                "schema_version": request.schema_version
            }
        });
        if self.config.background_mode {
            body["background"] = json!(true);
        }
        let mut web_search = json!({
            "type": "web_search",
            "search_context_size": self.config.search_context_size.as_str()
        });
        if !self.config.source_policy.allowed_domains.is_empty() {
            web_search["filters"] = json!({
                "allowed_domains": &self.config.source_policy.allowed_domains
            });
        }
        apply_token_limit(
            &mut body,
            self.config.token_limit_field,
            self.config.max_output_tokens,
        );
        body["tools"] = json!([web_search]);
        body["tool_choice"] = json!("auto");
        body["include"] = json!(["web_search_call.action.sources"]);
        body["max_tool_calls"] = json!(self.config.max_tool_calls);
        Ok(body)
    }

    fn build_plain_text_request_body(
        &self,
        request: &PlainTextGatewayRequest,
        model: &str,
    ) -> Result<Value, GatewayError> {
        let mut messages = Vec::with_capacity(request.messages.len() + 1);
        messages.push(("system", request.static_instructions.as_str()));
        for message in &request.messages {
            messages.push((message.role.as_str(), message.content.as_str()));
        }
        let body = match self.config.api_protocol {
            ApiProtocol::Responses => json!({
                "model": model,
                "input": messages.into_iter().map(|(role, content)| json!({
                    "role": role,
                    "content": [{"type": "input_text", "text": content}]
                })).collect::<Vec<_>>()
            }),
            ApiProtocol::ChatCompletions => json!({
                "model": model,
                "messages": messages.into_iter().map(|(role, content)| json!({
                    "role": role,
                    "content": content
                })).collect::<Vec<_>>()
            }),
        };
        Ok(body)
    }

    fn build_structured_request_body(
        &self,
        request: &StructuredGatewayRequest,
        model: &str,
        include_web_search: bool,
    ) -> Result<Value, GatewayError> {
        let mut provider_schema = request.schema.clone();
        if let Value::Object(schema) = &mut provider_schema {
            schema.remove("$schema");
            schema.remove("$id");
        }
        let schema_text = serde_json::to_string_pretty(&provider_schema).map_err(|error| {
            GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                format!("API协作Schema无法序列化：{error}"),
                false,
                "修正返回Schema后重试",
            )
        })?;
        let input_text = serde_json::to_string_pretty(&request.input).map_err(|error| {
            GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                format!("API协作输入无法序列化：{error}"),
                false,
                "修正输入内容后重试",
            )
        })?;
        let availability_note = if request.enable_web_search {
            "If the provider/model has native current-information or browsing capability, use it and include public source URLs in source_urls. If it does not, explicitly list the missing current information and do not invent citations."
        } else {
            "Do not claim that external browsing was performed."
        };
        let system_prompt = format!(
            "{}

Return only one JSON object. Do not use Markdown fences or explanatory text outside JSON. The JSON must follow this schema:
{}

{}",
            request.static_instructions, schema_text, availability_note
        );
        let mut body = match self.config.api_protocol {
            ApiProtocol::Responses => json!({
                "model": model,
                "input": [
                    {
                        "role": "system",
                        "content": [{"type": "input_text", "text": system_prompt}]
                    },
                    {
                        "role": "user",
                        "content": [{"type": "input_text", "text": input_text}]
                    }
                ]
            }),
            ApiProtocol::ChatCompletions => json!({
                "model": model,
                "messages": [
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": input_text}
                ]
            }),
        };
        if include_web_search && self.config.api_protocol == ApiProtocol::Responses {
            let mut web_search = json!({
                "type": "web_search",
                "search_context_size": self.config.search_context_size.as_str()
            });
            if !self.config.source_policy.allowed_domains.is_empty() {
                web_search["filters"] = json!({
                    "allowed_domains": &self.config.source_policy.allowed_domains
                });
            }
            body["tools"] = json!([web_search]);
            body["tool_choice"] = json!("auto");
            body["include"] = json!(["web_search_call.action.sources"]);
            body["max_tool_calls"] = json!(self.config.max_tool_calls);
        }
        Ok(body)
    }

    fn check_plain_text_budget(
        &self,
        request: &PlainTextGatewayRequest,
    ) -> Result<(), GatewayError> {
        if !request.daily_spend_usd.is_finite()
            || request.daily_spend_usd < 0.0
            || !request.monthly_spend_usd.is_finite()
            || request.monthly_spend_usd < 0.0
        {
            return Err(budget_error("数据库返回了无效的AI问答预算用量"));
        }
        let mut models = vec![self.model_for_operation(request.operation)];
        if let Some(fallback) = self.config.fallback_model.as_deref() {
            if !models.contains(&fallback) {
                models.push(fallback);
            }
        }
        let input_tokens = serde_json::to_string(request)
            .map(|value| (value.chars().count() as f64 / 4.0).ceil())
            .unwrap_or(0.0);
        let estimate = models
            .into_iter()
            .filter_map(|model| self.pricing_for_model(model))
            .map(|pricing| {
                input_tokens / 1_000_000.0 * pricing.input_usd_per_million
                    + self.config.max_output_tokens as f64 / 1_000_000.0
                        * pricing.output_usd_per_million
            })
            .sum::<f64>()
            * self.max_attempts_per_model() as f64;
        if let Some(limit) = self.config.budget.daily_budget_usd {
            if request.daily_spend_usd + estimate > limit {
                return Err(budget_error(format!(
                    "本次请求可能使今日成本超过{limit:.6}美元预算"
                )));
            }
        }
        if let Some(limit) = self.config.budget.monthly_budget_usd {
            if request.monthly_spend_usd + estimate > limit {
                return Err(budget_error(format!(
                    "本次请求可能使本月成本超过{limit:.6}美元预算"
                )));
            }
        }
        if let Some(limit) = self.config.budget.per_request_max_usd {
            if estimate > limit {
                return Err(budget_error(format!(
                    "本次请求成本上界{estimate:.6}美元超过单次预算{limit:.6}美元"
                )));
            }
        }
        Ok(())
    }

    fn estimate_plain_text_actual_cost(&self, response: &PlainTextGatewayResponse) -> Option<f64> {
        let pricing = self.pricing_for_model(&response.model_id)?;
        let uncached = response
            .usage
            .input_tokens
            .saturating_sub(response.usage.cached_input_tokens);
        Some(
            uncached as f64 / 1_000_000.0 * pricing.input_usd_per_million
                + response.usage.cached_input_tokens as f64 / 1_000_000.0
                    * pricing.cached_input_usd_per_million
                + response.usage.output_tokens as f64 / 1_000_000.0
                    * pricing.output_usd_per_million,
        )
    }

    fn check_structured_budget(
        &self,
        request: &StructuredGatewayRequest,
    ) -> Result<(), GatewayError> {
        if !request.daily_spend_usd.is_finite()
            || request.daily_spend_usd < 0.0
            || !request.monthly_spend_usd.is_finite()
            || request.monthly_spend_usd < 0.0
        {
            return Err(budget_error("数据库返回了无效的API预算用量"));
        }
        let mut models = vec![self.model_for_operation(request.operation)];
        if let Some(fallback) = self.config.fallback_model.as_deref() {
            if !models.contains(&fallback) {
                models.push(fallback);
            }
        }
        let input_tokens = serde_json::to_string(request)
            .map(|value| (value.chars().count() as f64 / 4.0).ceil())
            .unwrap_or(0.0);
        let search_cost = if request.enable_web_search
            && self.config.api_protocol == ApiProtocol::Responses
            && self
                .config
                .api_workspace_web_search_mode
                .allows_responses_web_search()
        {
            self.config.max_tool_calls as f64 * self.config.budget.web_search_usd_per_call
        } else {
            0.0
        };
        let automatic_fallback_attempt = request.enable_web_search
            && self.config.api_protocol == ApiProtocol::Responses
            && self
                .config
                .api_workspace_web_search_mode
                .allows_automatic_fallback();
        let attempts_per_model = self
            .max_attempts_per_model()
            .saturating_add(u32::from(automatic_fallback_attempt));
        let estimate = models
            .into_iter()
            .filter_map(|model| self.pricing_for_model(model))
            .map(|pricing| {
                input_tokens / 1_000_000.0 * pricing.input_usd_per_million
                    + self.config.max_output_tokens as f64 / 1_000_000.0
                        * pricing.output_usd_per_million
                    + search_cost
            })
            .sum::<f64>()
            * attempts_per_model as f64;
        if let Some(limit) = self.config.budget.daily_budget_usd {
            if request.daily_spend_usd + estimate > limit {
                return Err(budget_error(format!(
                    "本次请求可能使今日成本超过{limit:.6}美元预算"
                )));
            }
        }
        if let Some(limit) = self.config.budget.monthly_budget_usd {
            if request.monthly_spend_usd + estimate > limit {
                return Err(budget_error(format!(
                    "本次请求可能使本月成本超过{limit:.6}美元预算"
                )));
            }
        }
        if let Some(limit) = self.config.budget.per_request_max_usd {
            if estimate > limit {
                return Err(budget_error(format!(
                    "本次请求成本上界{estimate:.6}美元超过单次预算{limit:.6}美元"
                )));
            }
        }
        Ok(())
    }

    fn estimate_structured_actual_cost(&self, response: &StructuredGatewayResponse) -> Option<f64> {
        let pricing = self.pricing_for_model(&response.model_id)?;
        let uncached = response
            .usage
            .input_tokens
            .saturating_sub(response.usage.cached_input_tokens);
        Some(
            uncached as f64 / 1_000_000.0 * pricing.input_usd_per_million
                + response.usage.cached_input_tokens as f64 / 1_000_000.0
                    * pricing.cached_input_usd_per_million
                + response.usage.output_tokens as f64 / 1_000_000.0
                    * pricing.output_usd_per_million
                + response.search_call_count as f64 * self.config.budget.web_search_usd_per_call,
        )
    }

    fn model_for_operation(&self, operation: GatewayOperation) -> &str {
        match operation {
            GatewayOperation::Research => &self.config.research_model,
            GatewayOperation::Extraction => &self.config.extraction_model,
        }
    }

    fn check_budget(&self, request: &GatewayRequest) -> Result<(), GatewayError> {
        if !request.daily_spend_usd.is_finite()
            || request.daily_spend_usd < 0.0
            || !request.monthly_spend_usd.is_finite()
            || request.monthly_spend_usd < 0.0
        {
            return Err(budget_error("数据库返回了无效的OpenAI预算用量"));
        }
        let mut models = vec![self.model_for_operation(request.operation)];
        if let Some(fallback) = self.config.fallback_model.as_deref() {
            if !models.contains(&fallback) {
                models.push(fallback);
            }
        }
        let attempts_per_model = self.max_attempts_per_model() as f64;
        let estimate = models
            .into_iter()
            .filter_map(|model| self.estimate_request_ceiling(request, model))
            .map(|ceiling| ceiling * attempts_per_model)
            .sum::<f64>();

        if let Some(limit) = self.config.budget.daily_budget_usd {
            if request.daily_spend_usd + estimate > limit {
                return Err(budget_error(format!(
                    "本次请求可能使今日成本超过{limit:.6}美元预算"
                )));
            }
        }
        if let Some(limit) = self.config.budget.monthly_budget_usd {
            if request.monthly_spend_usd + estimate > limit {
                return Err(budget_error(format!(
                    "本次请求可能使本月成本超过{limit:.6}美元预算"
                )));
            }
        }
        if let Some(limit) = self.config.budget.per_request_max_usd {
            if estimate > limit {
                return Err(budget_error(format!(
                    "本次请求成本上界{estimate:.6}美元超过单次预算{limit:.6}美元"
                )));
            }
        }
        Ok(())
    }

    fn estimate_request_ceiling(&self, request: &GatewayRequest, model: &str) -> Option<f64> {
        let pricing = self.pricing_for_model(model)?;
        let serialized = serde_json::to_string(request).ok()?;
        let estimated_input_tokens = (serialized.chars().count() as f64 / 4.0).ceil();
        let search_cost =
            self.config.max_tool_calls as f64 * self.config.budget.web_search_usd_per_call;
        Some(
            estimated_input_tokens / 1_000_000.0 * pricing.input_usd_per_million
                + self.config.max_output_tokens as f64 / 1_000_000.0
                    * pricing.output_usd_per_million
                + search_cost,
        )
    }

    fn max_attempts_per_model(&self) -> u32 {
        self.config.max_retries.saturating_add(1)
    }

    fn estimate_actual_cost(&self, response: &GatewayResponse) -> Option<f64> {
        let pricing = self.pricing_for_model(&response.model_id)?;
        let uncached = response
            .usage
            .input_tokens
            .saturating_sub(response.usage.cached_input_tokens);
        Some(
            uncached as f64 / 1_000_000.0 * pricing.input_usd_per_million
                + response.usage.cached_input_tokens as f64 / 1_000_000.0
                    * pricing.cached_input_usd_per_million
                + response.usage.output_tokens as f64 / 1_000_000.0
                    * pricing.output_usd_per_million
                + response.search_call_count as f64 * self.config.budget.web_search_usd_per_call,
        )
    }

    fn pricing_for_model(&self, model: &str) -> Option<&ModelPricing> {
        self.config.budget.model_pricing.get(model).or_else(|| {
            self.config
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

    async fn check_circuit(&self) -> Result<(), GatewayError> {
        let mut state = self.circuit.lock().await;
        if let Some(open_until) = state.open_until {
            if Instant::now() < open_until {
                return Err(GatewayError::new(
                    GatewayErrorCategory::CircuitOpen,
                    "OpenAI研究网关因连续失败已暂时熔断",
                    true,
                    "等待熔断窗口结束；历史记录和手工事实仍可继续使用",
                ));
            }
            state.open_until = None;
            state.consecutive_failures = 0;
        }
        Ok(())
    }

    async fn record_success(&self) {
        let mut state = self.circuit.lock().await;
        state.consecutive_failures = 0;
        state.open_until = None;
    }

    async fn record_failure(&self, error: &GatewayError) {
        if !matches!(
            error.category,
            GatewayErrorCategory::Network
                | GatewayErrorCategory::Timeout
                | GatewayErrorCategory::RateLimit
                | GatewayErrorCategory::ProviderUnavailable
        ) {
            return;
        }
        let mut state = self.circuit.lock().await;
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        if state.consecutive_failures >= self.config.circuit_breaker.consecutive_failure_threshold {
            state.open_until = Some(
                Instant::now() + Duration::from_secs(self.config.circuit_breaker.open_seconds),
            );
        }
    }
}

fn unsupported_structured_web_search(status: u16, body: &Value) -> bool {
    if !matches!(status, 400 | 404 | 415 | 422) {
        return false;
    }
    let text = body.to_string().to_ascii_lowercase();
    let mentions_unsupported = [
        "unsupported parameter",
        "unsupported field",
        "unknown parameter",
        "unknown field",
        "unrecognized parameter",
        "not supported",
    ]
    .iter()
    .any(|needle| text.contains(needle));
    let mentions_web_field = [
        "web_search",
        "tools",
        "tool_choice",
        "max_tool_calls",
        "include",
    ]
    .iter()
    .any(|needle| text.contains(needle));
    mentions_unsupported && mentions_web_field
}

fn apply_token_limit(body: &mut Value, field: TokenLimitField, value: u32) {
    if let Value::Object(object) = body {
        object.remove("max_output_tokens");
        object.remove("max_tokens");
        object.insert(field.as_str().to_string(), json!(value));
    }
}

fn elapsed_millis(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn response_id_from_body(body: &Value) -> Option<String> {
    body.get("id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn retry_delay(base_ms: u64, attempt_number: u32) -> Duration {
    let exponent = attempt_number.saturating_sub(1).min(10);
    Duration::from_millis(base_ms.saturating_mul(1u64 << exponent))
}

fn fingerprint(value: &Value) -> Result<String, GatewayError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            format!("OpenAI请求无法计算指纹：{error}"),
            false,
            "修正请求载荷后重试",
        )
    })?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn map_reqwest_error(error: reqwest::Error) -> GatewayError {
    if error.is_timeout() {
        GatewayError::new(
            GatewayErrorCategory::Timeout,
            "OpenAI请求超时",
            true,
            "按退避策略重试；持续超时则缩小搜索范围或提高超时配置",
        )
    } else {
        network_error(format!("OpenAI网络请求失败：{error}"))
    }
}

fn network_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::Network,
        message,
        true,
        "检查网络连接后按幂等键重试；应用历史记录和手工事实不受影响",
    )
}

fn cancelled_error() -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::Cancelled,
        "OpenAI研究任务已取消",
        false,
        "可从已保存的研究任务重新发起；已存在证据和快照不会被删除",
    )
}

fn budget_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::BudgetExceeded,
        message,
        false,
        "调整预算配置或等待下一预算周期；不得绕过预算生成伪完成状态",
    )
}
