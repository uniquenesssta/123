use crate::{GatewayError, GatewayErrorCategory};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApiProtocol {
    #[default]
    Responses,
    ChatCompletions,
}

impl ApiProtocol {
    pub const fn endpoint_suffix(self) -> &'static str {
        match self {
            Self::Responses => "/responses",
            Self::ChatCompletions => "/chat/completions",
        }
    }

    pub const fn default_token_limit_field(self) -> TokenLimitField {
        match self {
            Self::Responses => TokenLimitField::MaxOutputTokens,
            Self::ChatCompletions => TokenLimitField::MaxTokens,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TokenLimitField {
    #[default]
    MaxOutputTokens,
    MaxTokens,
}

impl TokenLimitField {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MaxOutputTokens => "max_output_tokens",
            Self::MaxTokens => "max_tokens",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ApiWorkspaceWebSearchMode {
    Disabled,
    #[default]
    Auto,
    ResponsesWebSearch,
}

impl ApiWorkspaceWebSearchMode {
    pub const fn allows_responses_web_search(self) -> bool {
        matches!(self, Self::Auto | Self::ResponsesWebSearch)
    }

    pub const fn allows_automatic_fallback(self) -> bool {
        matches!(self, Self::Auto)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
}

impl ReasoningEffort {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchContextSize {
    Low,
    Medium,
    High,
}

impl SearchContextSize {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelPricing {
    pub input_usd_per_million: f64,
    pub cached_input_usd_per_million: f64,
    pub output_usd_per_million: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BudgetConfig {
    #[serde(default)]
    pub daily_budget_usd: Option<f64>,
    #[serde(default)]
    pub monthly_budget_usd: Option<f64>,
    #[serde(default)]
    pub per_request_max_usd: Option<f64>,
    #[serde(default)]
    pub web_search_usd_per_call: f64,
    #[serde(default)]
    pub model_pricing: BTreeMap<String, ModelPricing>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CircuitBreakerConfig {
    pub consecutive_failure_threshold: u32,
    pub open_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialMode {
    WindowsCredentialManager,
    ServerEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialConfig {
    pub mode: CredentialMode,
    #[serde(default = "default_credential_target")]
    pub credential_target: String,
    #[serde(default = "default_environment_variable")]
    pub environment_variable: String,
    #[serde(default = "default_deployment_mode")]
    pub deployment_mode: String,
}

fn default_credential_target() -> String {
    "football-match-model-platform/openai".to_string()
}

fn default_environment_variable() -> String {
    "OPENAI_API_KEY".to_string()
}

fn default_deployment_mode() -> String {
    "local_desktop".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourcePolicy {
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub blocked_domains: Vec<String>,
    #[serde(default)]
    pub prohibited_fact_keys: Vec<String>,
    #[serde(default)]
    pub prohibited_content_terms: Vec<String>,
    #[serde(default = "default_https_only")]
    pub https_only: bool,
}

fn default_https_only() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GatewayConfig {
    pub api_base_url: String,
    #[serde(default)]
    pub api_protocol: ApiProtocol,
    #[serde(default)]
    pub request_endpoint: Option<String>,
    #[serde(default)]
    pub token_limit_field: TokenLimitField,
    #[serde(default)]
    pub api_workspace_web_search_mode: ApiWorkspaceWebSearchMode,
    pub research_model: String,
    pub extraction_model: String,
    #[serde(default)]
    pub fallback_model: Option<String>,
    pub reasoning_effort: ReasoningEffort,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub retry_base_delay_ms: u64,
    pub max_concurrency: usize,
    pub max_output_tokens: u32,
    pub max_tool_calls: u32,
    pub search_context_size: SearchContextSize,
    #[serde(default)]
    pub background_mode: bool,
    #[serde(default)]
    pub zero_data_retention_required: bool,
    #[serde(default)]
    pub store: bool,
    pub credentials: CredentialConfig,
    pub source_policy: SourcePolicy,
    pub budget: BudgetConfig,
    pub circuit_breaker: CircuitBreakerConfig,
}

impl GatewayConfig {
    pub fn validate(&self) -> Result<(), GatewayError> {
        let api_url = Url::parse(&self.api_base_url)
            .map_err(|_| config_error("兼容 API基础地址不是有效URL"))?;
        let is_https = api_url.scheme() == "https";
        let is_loopback_host = api_url
            .host_str()
            .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"));
        let is_loopback_http = api_url.scheme() == "http" && is_loopback_host;
        if !is_https && !is_loopback_http {
            return Err(config_error("API地址必须使用HTTPS；仅测试允许本机回环地址"));
        }
        if !api_url.username().is_empty() || api_url.password().is_some() {
            return Err(config_error("兼容 API基础地址不能内嵌用户名或密码"));
        }
        if api_url.query().is_some() || api_url.fragment().is_some() {
            return Err(config_error("兼容 API基础地址不能包含查询参数或片段"));
        }
        let endpoint = Url::parse(&self.request_endpoint())
            .map_err(|_| config_error("兼容 API请求端点不是有效URL"))?;
        let endpoint_is_loopback = endpoint
            .host_str()
            .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"));
        if endpoint.scheme() != "https" && !(endpoint.scheme() == "http" && endpoint_is_loopback) {
            return Err(config_error(
                "兼容 API请求端点必须使用HTTPS；仅测试允许本机回环地址",
            ));
        }
        if !endpoint.username().is_empty() || endpoint.password().is_some() {
            return Err(config_error("兼容 API请求端点不能内嵌用户名或密码"));
        }
        if endpoint.query().is_some() || endpoint.fragment().is_some() {
            return Err(config_error("兼容 API请求端点不能包含查询参数或片段"));
        }
        if self.api_workspace_web_search_mode == ApiWorkspaceWebSearchMode::ResponsesWebSearch
            && self.api_protocol != ApiProtocol::Responses
        {
            return Err(config_error(
                "强制 Web Search 仅支持 Responses 协议；Chat Completions 请使用自动或关闭联网",
            ));
        }
        if !endpoint
            .path()
            .trim_end_matches('/')
            .ends_with(self.api_protocol.endpoint_suffix())
        {
            return Err(config_error("兼容 API请求端点与所选协议不一致"));
        }
        if self.research_model.trim().is_empty() || self.extraction_model.trim().is_empty() {
            return Err(config_error("research_model和extraction_model不能为空"));
        }
        if self
            .fallback_model
            .as_deref()
            .is_some_and(|fallback| fallback.trim().is_empty())
        {
            return Err(config_error("fallback_model不能是空字符串"));
        }
        if self.timeout_seconds == 0 || self.max_concurrency == 0 {
            return Err(config_error("超时和最大并发必须大于0"));
        }
        if self.max_retries > 10 || self.retry_base_delay_ms == 0 {
            return Err(config_error("最大重试不能超过10次，退避基础延迟必须大于0"));
        }
        if self.max_output_tokens == 0 || self.max_tool_calls == 0 {
            return Err(config_error("输出Token和工具调用上限必须大于0"));
        }
        if self.background_mode && !self.store {
            return Err(config_error(
                "Background mode需要store=true以支持response_id轮询和断线恢复",
            ));
        }
        if self.zero_data_retention_required && (self.background_mode || self.store) {
            return Err(config_error(
                "零数据保留模式必须使用background=false且store=false",
            ));
        }
        match self.credentials.mode {
            CredentialMode::WindowsCredentialManager => {
                if self.credentials.deployment_mode != "local_desktop" {
                    return Err(config_error("Windows凭据管理器模式仅允许local_desktop部署"));
                }
            }
            CredentialMode::ServerEnvironment => {
                if self.credentials.deployment_mode != "server" {
                    return Err(config_error(
                        "环境变量密钥只允许显式server部署，桌面端不得读取环境变量密钥",
                    ));
                }
            }
        }
        validate_domains(&self.source_policy.allowed_domains)?;
        validate_domains(&self.source_policy.blocked_domains)?;
        if self.source_policy.allowed_domains.len() > 100 {
            return Err(config_error("Web Search允许域名不能超过100个"));
        }
        let allowed: BTreeSet<_> = self.source_policy.allowed_domains.iter().collect();
        let blocked: BTreeSet<_> = self.source_policy.blocked_domains.iter().collect();
        if !allowed.is_disjoint(&blocked) {
            return Err(config_error("来源域名不能同时出现在允许和禁止列表"));
        }
        validate_budget(self)?;
        validate_model_pricing(self)?;
        if self.circuit_breaker.consecutive_failure_threshold == 0
            || self.circuit_breaker.open_seconds == 0
        {
            return Err(config_error("熔断阈值和开启时间必须大于0"));
        }
        Ok(())
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }

    pub fn request_endpoint(&self) -> String {
        self.request_endpoint
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| {
                format!(
                    "{}{}",
                    self.api_base_url.trim_end_matches('/'),
                    self.api_protocol.endpoint_suffix()
                )
            })
    }

    pub fn responses_endpoint(&self) -> String {
        self.request_endpoint()
    }
}

fn validate_domains(domains: &[String]) -> Result<(), GatewayError> {
    for domain in domains {
        let normalized = domain.trim().trim_start_matches("*.");
        if normalized.is_empty()
            || normalized.contains('/')
            || normalized.contains(':')
            || !normalized.contains('.')
        {
            return Err(config_error(format!("无效来源域名：{domain}")));
        }
    }
    Ok(())
}

fn validate_budget(config: &GatewayConfig) -> Result<(), GatewayError> {
    for value in [
        config.budget.daily_budget_usd,
        config.budget.monthly_budget_usd,
        config.budget.per_request_max_usd,
    ]
    .into_iter()
    .flatten()
    {
        if !value.is_finite() || value <= 0.0 {
            return Err(config_error("预算值必须是大于0的有限数"));
        }
    }
    if !config.budget.web_search_usd_per_call.is_finite()
        || config.budget.web_search_usd_per_call < 0.0
    {
        return Err(config_error("Web Search单次费用必须是非负有限数"));
    }
    for (model, pricing) in &config.budget.model_pricing {
        if model.trim().is_empty()
            || [
                pricing.input_usd_per_million,
                pricing.cached_input_usd_per_million,
                pricing.output_usd_per_million,
            ]
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(config_error("模型价格配置无效"));
        }
    }
    Ok(())
}

fn validate_model_pricing(config: &GatewayConfig) -> Result<(), GatewayError> {
    let needs_pricing = config.budget.daily_budget_usd.is_some()
        || config.budget.monthly_budget_usd.is_some()
        || config.budget.per_request_max_usd.is_some()
        || !config.budget.model_pricing.is_empty();
    if !needs_pricing {
        return Ok(());
    }
    let mut models = BTreeSet::from([
        config.research_model.as_str(),
        config.extraction_model.as_str(),
    ]);
    if let Some(fallback) = config.fallback_model.as_deref() {
        models.insert(fallback);
    }
    for model in models {
        if !config.budget.model_pricing.contains_key(model) {
            return Err(config_error(format!(
                "模型{model}缺少成本价格配置，无法执行预算门禁和成本审计"
            )));
        }
    }
    Ok(())
}

fn config_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::InvalidConfiguration,
        message,
        false,
        "检查OpenAI研究网关配置后重试",
    )
}
