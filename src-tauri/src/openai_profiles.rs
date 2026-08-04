use crate::file_store::write_atomic;
use chrono::{DateTime, Utc};
use football_research_gateway::{
    delete_windows_api_key, parse_api_example, save_windows_api_key, test_openai_connection,
    windows_api_key_exists, ApiProtocol, ApiWorkspaceWebSearchMode, CredentialMode,
    DefaultApiKeyProvider, GatewayConfig, OpenAiConnectionTest, OpenAiResearchGateway,
    ReasoningEffort, ReqwestTransport, SearchContextSize, TokenLimitField,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use zeroize::Zeroize;

const STORE_VERSION: u32 = 1;
const CREDENTIAL_PREFIX: &str = "football-match-model-platform/openai";
const DEFAULT_CONFIG: &str = include_str!("../resources/research/openai_gateway.json");

#[derive(Clone, Serialize, Deserialize)]
pub struct OpenAiProfileDraft {
    pub id: Option<String>,
    pub name: String,
    pub api_key: Option<String>,
    pub api_base_url: String,
    pub api_protocol: ApiProtocol,
    pub api_endpoint: String,
    pub token_limit_field: TokenLimitField,
    #[serde(default)]
    pub api_workspace_web_search_mode: ApiWorkspaceWebSearchMode,
    pub api_example_template: Option<String>,
    pub research_model: String,
    pub extraction_model: String,
    pub fallback_model: Option<String>,
    pub reasoning_effort: ReasoningEffort,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub max_concurrency: usize,
    pub max_output_tokens: u32,
    pub max_tool_calls: u32,
    pub search_context_size: SearchContextSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiProfileSummary {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub api_protocol: ApiProtocol,
    pub api_endpoint: String,
    pub token_limit_field: TokenLimitField,
    pub api_workspace_web_search_mode: ApiWorkspaceWebSearchMode,
    pub api_example_template: Option<String>,
    pub formal_research_candidate: bool,
    pub is_active: bool,
    pub has_api_key: bool,
    pub api_key_mask: Option<String>,
    pub api_base_url: String,
    pub research_model: String,
    pub extraction_model: String,
    pub fallback_model: Option<String>,
    pub reasoning_effort: ReasoningEffort,
    pub timeout_seconds: u64,
    pub max_retries: u32,
    pub max_concurrency: usize,
    pub max_output_tokens: u32,
    pub max_tool_calls: u32,
    pub search_context_size: SearchContextSize,
    pub last_test_status: String,
    pub last_test_message: Option<String>,
    pub last_tested_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiProfilesState {
    pub active_profile_id: Option<String>,
    pub profiles: Vec<OpenAiProfileSummary>,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiProfileTestResult {
    pub profile_id: String,
    pub profile_name: String,
    pub model_id: String,
    pub protocol: ApiProtocol,
    pub endpoint_url: String,
    pub latency_ms: u64,
    pub provider_request_id: Option<String>,
    pub tested_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredOpenAiProfile {
    id: Uuid,
    name: String,
    api_base_url: String,
    #[serde(default)]
    api_protocol: ApiProtocol,
    #[serde(default)]
    api_endpoint: String,
    #[serde(default)]
    token_limit_field: TokenLimitField,
    #[serde(default)]
    api_workspace_web_search_mode: ApiWorkspaceWebSearchMode,
    #[serde(default)]
    api_example_template: Option<String>,
    research_model: String,
    extraction_model: String,
    fallback_model: Option<String>,
    reasoning_effort: ReasoningEffort,
    timeout_seconds: u64,
    max_retries: u32,
    max_concurrency: usize,
    max_output_tokens: u32,
    max_tool_calls: u32,
    search_context_size: SearchContextSize,
    last_test_status: String,
    last_test_message: Option<String>,
    last_tested_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredOpenAiProfiles {
    version: u32,
    active_profile_id: Option<Uuid>,
    profiles: Vec<StoredOpenAiProfile>,
}

pub struct OpenAiProfileStore {
    path: PathBuf,
    state: Mutex<StoredOpenAiProfiles>,
}

impl OpenAiProfileStore {
    pub fn new(path: PathBuf) -> Self {
        let mut state = load_state(&path);
        if state.profiles.is_empty() {
            state.profiles.push(default_profile());
        }
        if state.active_profile_id.is_none() {
            state.active_profile_id = state.profiles.first().map(|profile| profile.id);
        }
        let store = Self {
            path,
            state: Mutex::new(state),
        };
        let _ = store.persist();
        store
    }

    pub fn list(&self) -> Result<OpenAiProfilesState, String> {
        let state = self.lock()?;
        let profiles = state
            .profiles
            .iter()
            .map(|profile| self.summary(profile, state.active_profile_id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(OpenAiProfilesState {
            active_profile_id: state.active_profile_id.map(|id| id.to_string()),
            profiles,
            config_path: self.path.display().to_string(),
        })
    }

    pub fn save(&self, mut draft: OpenAiProfileDraft) -> Result<OpenAiProfileSummary, String> {
        normalize_draft(&mut draft)?;
        let requested_id = draft
            .id
            .as_deref()
            .map(|value| {
                Uuid::parse_str(value).map_err(|error| format!("兼容 API配置ID无效：{error}"))
            })
            .transpose()?;
        let api_key_changed = draft
            .api_key
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());
        let mut state = self.lock()?;
        if state.profiles.iter().any(|profile| {
            Some(profile.id) != requested_id && profile.name.eq_ignore_ascii_case(&draft.name)
        }) {
            return Err("兼容 API配置名称已存在，请使用不同名称".to_string());
        }

        let now = Utc::now();
        let profile_id = requested_id.unwrap_or_else(Uuid::new_v4);
        let existing_index = state
            .profiles
            .iter()
            .position(|profile| profile.id == profile_id);
        if requested_id.is_some() && existing_index.is_none() {
            return Err("需要编辑的兼容 API配置不存在".to_string());
        }
        let created_at = existing_index
            .map(|index| state.profiles[index].created_at)
            .unwrap_or(now);
        let connection_changed = existing_index
            .map(|index| {
                api_key_changed || connection_settings_changed(&state.profiles[index], &draft)
            })
            .unwrap_or(true);
        let previous_test = existing_index.filter(|_| !connection_changed).map(|index| {
            let profile = &state.profiles[index];
            (
                profile.last_test_status.clone(),
                profile.last_test_message.clone(),
                profile.last_tested_at,
            )
        });
        let mut api_key = draft.api_key.take();
        let profile = StoredOpenAiProfile {
            id: profile_id,
            name: draft.name,
            api_base_url: draft.api_base_url,
            api_protocol: draft.api_protocol,
            api_endpoint: draft.api_endpoint,
            token_limit_field: draft.token_limit_field,
            api_workspace_web_search_mode: draft.api_workspace_web_search_mode,
            api_example_template: draft.api_example_template,
            research_model: draft.research_model,
            extraction_model: draft.extraction_model,
            fallback_model: draft.fallback_model,
            reasoning_effort: draft.reasoning_effort,
            timeout_seconds: draft.timeout_seconds,
            max_retries: draft.max_retries,
            max_concurrency: draft.max_concurrency,
            max_output_tokens: draft.max_output_tokens,
            max_tool_calls: draft.max_tool_calls,
            search_context_size: draft.search_context_size,
            last_test_status: previous_test
                .as_ref()
                .map(|value| value.0.clone())
                .unwrap_or_else(|| "untested".to_string()),
            last_test_message: previous_test.as_ref().and_then(|value| value.1.clone()),
            last_tested_at: previous_test.and_then(|value| value.2),
            created_at,
            updated_at: now,
        };
        validate_profile(&profile)?;

        let previous_state = state.clone();
        if let Some(index) = existing_index {
            state.profiles[index] = profile.clone();
        } else {
            state.profiles.push(profile.clone());
        }
        if state.active_profile_id.is_none() {
            state.active_profile_id = Some(profile_id);
        }
        persist_state(&self.path, &state)?;

        if let Some(value) = api_key.as_mut() {
            if !value.trim().is_empty() {
                let secret = std::mem::take(value);
                if let Err(error) = save_windows_api_key(&credential_target(profile_id), secret) {
                    *state = previous_state;
                    let rollback = persist_state(&self.path, &state);
                    value.zeroize();
                    return match rollback {
                        Ok(()) => Err(error.to_string()),
                        Err(rollback_error) => Err(format!(
                            "{error}；同时无法回滚兼容 API配置元数据：{rollback_error}"
                        )),
                    };
                }
            }
            value.zeroize();
        }
        self.summary(&profile, state.active_profile_id)
    }

    pub fn set_active(&self, profile_id: &str) -> Result<OpenAiProfilesState, String> {
        let profile_id = parse_profile_id(profile_id)?;
        let mut state = self.lock()?;
        if !state
            .profiles
            .iter()
            .any(|profile| profile.id == profile_id)
        {
            return Err("兼容 API配置不存在".to_string());
        }
        if !windows_api_key_exists(&credential_target(profile_id))
            .map_err(|error| error.to_string())?
        {
            return Err("该兼容 API配置尚未保存API Key，不能设为当前使用配置".to_string());
        }
        state.active_profile_id = Some(profile_id);
        persist_state(&self.path, &state)?;
        drop(state);
        self.list()
    }

    pub fn delete(&self, profile_id: &str) -> Result<OpenAiProfilesState, String> {
        let profile_id = parse_profile_id(profile_id)?;
        let mut state = self.lock()?;
        if !state
            .profiles
            .iter()
            .any(|profile| profile.id == profile_id)
        {
            return Err("兼容 API配置不存在".to_string());
        }
        let previous_state = state.clone();
        state.profiles.retain(|profile| profile.id != profile_id);
        if state.active_profile_id == Some(profile_id) {
            state.active_profile_id = state.profiles.first().map(|profile| profile.id);
        }
        persist_state(&self.path, &state)?;
        if let Err(error) = delete_windows_api_key(&credential_target(profile_id)) {
            *state = previous_state;
            let rollback = persist_state(&self.path, &state);
            return match rollback {
                Ok(()) => Err(error.to_string()),
                Err(rollback_error) => Err(format!(
                    "{error}；同时无法回滚兼容 API配置元数据：{rollback_error}"
                )),
            };
        }
        drop(state);
        self.list()
    }

    pub fn clear_key(&self, profile_id: &str) -> Result<OpenAiProfilesState, String> {
        let profile_id = parse_profile_id(profile_id)?;
        let mut state = self.lock()?;
        let previous_state = state.clone();
        let profile = state
            .profiles
            .iter_mut()
            .find(|profile| profile.id == profile_id)
            .ok_or_else(|| "兼容 API配置不存在".to_string())?;
        profile.last_test_status = "untested".to_string();
        profile.last_test_message = None;
        profile.last_tested_at = None;
        profile.updated_at = Utc::now();
        persist_state(&self.path, &state)?;
        if let Err(error) = delete_windows_api_key(&credential_target(profile_id)) {
            *state = previous_state;
            let rollback = persist_state(&self.path, &state);
            return match rollback {
                Ok(()) => Err(error.to_string()),
                Err(rollback_error) => Err(format!(
                    "{error}；同时无法回滚兼容 API配置元数据：{rollback_error}"
                )),
            };
        }
        drop(state);
        self.list()
    }

    pub fn gateway_for(
        &self,
        profile_id: Option<&str>,
    ) -> Result<(OpenAiProfileSummary, OpenAiResearchGateway), String> {
        let state = self.lock()?;
        let selected_id = profile_id
            .map(parse_profile_id)
            .transpose()?
            .or(state.active_profile_id)
            .ok_or_else(|| "尚未选择兼容 API配置".to_string())?;
        let profile = state
            .profiles
            .iter()
            .find(|profile| profile.id == selected_id)
            .cloned()
            .ok_or_else(|| "兼容 API配置不存在".to_string())?;
        if !windows_api_key_exists(&credential_target(profile.id))
            .map_err(|error| error.to_string())?
        {
            return Err("该兼容 API配置尚未保存API Key".to_string());
        }
        let summary = self.summary(&profile, state.active_profile_id)?;
        let config = gateway_config(&profile)?;
        drop(state);
        let gateway = OpenAiResearchGateway::new(
            config,
            Arc::new(ReqwestTransport::new().map_err(|error| error.to_string())?),
            Arc::new(DefaultApiKeyProvider),
        )
        .map_err(|error| error.to_string())?;
        Ok((summary, gateway))
    }

    pub async fn test(&self, profile_id: &str) -> Result<OpenAiProfileTestResult, String> {
        let profile_id = parse_profile_id(profile_id)?;
        let (profile, config) = {
            let state = self.lock()?;
            let profile = state
                .profiles
                .iter()
                .find(|profile| profile.id == profile_id)
                .cloned()
                .ok_or_else(|| "兼容 API配置不存在".to_string())?;
            let config = gateway_config(&profile)?;
            (profile, config)
        };
        let transport = ReqwestTransport::new().map_err(|error| error.to_string())?;
        let result = test_openai_connection(
            &config,
            &DefaultApiKeyProvider,
            &transport,
            &profile.research_model,
        )
        .await;
        let tested_at = Utc::now();
        match result {
            Ok(OpenAiConnectionTest {
                model_id,
                protocol,
                endpoint_url,
                provider_request_id,
                latency_ms,
            }) => {
                self.update_test_state(
                    profile_id,
                    "success",
                    Some("连接与模型权限验证通过"),
                    tested_at,
                )?;
                Ok(OpenAiProfileTestResult {
                    profile_id: profile_id.to_string(),
                    profile_name: profile.name,
                    model_id,
                    protocol,
                    endpoint_url,
                    latency_ms,
                    provider_request_id,
                    tested_at,
                })
            }
            Err(error) => {
                self.update_test_state(profile_id, "failed", Some(&error.user_message), tested_at)?;
                Err(error.to_string())
            }
        }
    }

    fn update_test_state(
        &self,
        profile_id: Uuid,
        status: &str,
        message: Option<&str>,
        tested_at: DateTime<Utc>,
    ) -> Result<(), String> {
        let mut state = self.lock()?;
        let profile = state
            .profiles
            .iter_mut()
            .find(|profile| profile.id == profile_id)
            .ok_or_else(|| "兼容 API配置不存在".to_string())?;
        profile.last_test_status = status.to_string();
        profile.last_test_message = message.map(|value| truncate(value, 500));
        profile.last_tested_at = Some(tested_at);
        profile.updated_at = tested_at;
        persist_state(&self.path, &state)
    }

    fn summary(
        &self,
        profile: &StoredOpenAiProfile,
        active_profile_id: Option<Uuid>,
    ) -> Result<OpenAiProfileSummary, String> {
        let has_api_key = windows_api_key_exists(&credential_target(profile.id))
            .map_err(|error| error.to_string())?;
        Ok(OpenAiProfileSummary {
            id: profile.id.to_string(),
            name: profile.name.clone(),
            provider: "openai_compatible".to_string(),
            api_protocol: profile.api_protocol,
            api_endpoint: effective_endpoint(profile),
            token_limit_field: profile.token_limit_field,
            api_workspace_web_search_mode: profile.api_workspace_web_search_mode,
            api_example_template: profile.api_example_template.clone(),
            formal_research_candidate: profile.api_protocol == ApiProtocol::Responses,
            is_active: active_profile_id == Some(profile.id),
            has_api_key,
            api_key_mask: has_api_key.then(|| "••••••••••••••••".to_string()),
            api_base_url: profile.api_base_url.clone(),
            research_model: profile.research_model.clone(),
            extraction_model: profile.extraction_model.clone(),
            fallback_model: profile.fallback_model.clone(),
            reasoning_effort: profile.reasoning_effort,
            timeout_seconds: profile.timeout_seconds,
            max_retries: profile.max_retries,
            max_concurrency: profile.max_concurrency,
            max_output_tokens: profile.max_output_tokens,
            max_tool_calls: profile.max_tool_calls,
            search_context_size: profile.search_context_size,
            last_test_status: profile.last_test_status.clone(),
            last_test_message: profile.last_test_message.clone(),
            last_tested_at: profile.last_tested_at,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        })
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, StoredOpenAiProfiles>, String> {
        self.state
            .lock()
            .map_err(|_| "兼容 API配置暂时不可用：配置锁已损坏".to_string())
    }

    fn persist(&self) -> Result<(), String> {
        let state = self.lock()?;
        persist_state(&self.path, &state)
    }
}

fn load_state(path: &Path) -> StoredOpenAiProfiles {
    fs::read(path)
        .ok()
        .and_then(|content| serde_json::from_slice::<StoredOpenAiProfiles>(&content).ok())
        .filter(|state| state.version == STORE_VERSION)
        .unwrap_or_else(|| StoredOpenAiProfiles {
            version: STORE_VERSION,
            active_profile_id: None,
            profiles: Vec::new(),
        })
}

fn persist_state(path: &Path, state: &StoredOpenAiProfiles) -> Result<(), String> {
    let content = serde_json::to_vec_pretty(state)
        .map_err(|error| format!("无法序列化兼容 API配置：{error}"))?;
    write_atomic(path, &content, true)
}

fn default_profile() -> StoredOpenAiProfile {
    let config: GatewayConfig =
        serde_json::from_str(DEFAULT_CONFIG).expect("内置兼容 API研究网关配置必须有效");
    let now = Utc::now();
    let api_endpoint = config.request_endpoint();
    StoredOpenAiProfile {
        id: Uuid::new_v4(),
        name: "兼容 API 默认配置".to_string(),
        api_base_url: config.api_base_url,
        api_protocol: config.api_protocol,
        api_endpoint,
        token_limit_field: config.token_limit_field,
        api_workspace_web_search_mode: config.api_workspace_web_search_mode,
        api_example_template: None,
        research_model: config.research_model,
        extraction_model: config.extraction_model,
        fallback_model: config.fallback_model,
        reasoning_effort: config.reasoning_effort,
        timeout_seconds: config.timeout_seconds,
        max_retries: config.max_retries,
        max_concurrency: config.max_concurrency,
        max_output_tokens: config.max_output_tokens,
        max_tool_calls: config.max_tool_calls,
        search_context_size: config.search_context_size,
        last_test_status: "untested".to_string(),
        last_test_message: None,
        last_tested_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn normalize_draft(draft: &mut OpenAiProfileDraft) -> Result<(), String> {
    draft.name = draft.name.trim().to_string();
    draft.api_base_url = draft.api_base_url.trim().trim_end_matches('/').to_string();
    draft.api_endpoint = draft.api_endpoint.trim().trim_end_matches('/').to_string();
    draft.api_example_template = draft
        .api_example_template
        .take()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if let Some(example) = draft.api_example_template.clone() {
        let parsed = parse_api_example(&example, Some(draft.api_protocol))
            .map_err(|error| error.to_string())?;
        if draft
            .api_key
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            draft.api_key = parsed.selected.api_key.clone();
        }
        draft.api_example_template = Some(parsed.selected.sanitized_example);
    }
    draft.research_model = draft.research_model.trim().to_string();
    draft.extraction_model = draft.extraction_model.trim().to_string();
    draft.fallback_model = draft
        .fallback_model
        .take()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if draft.name.is_empty() || draft.name.chars().count() > 80 {
        return Err("兼容 API配置名称必须为1至80个字符".to_string());
    }
    for (label, model) in [
        ("研究模型", draft.research_model.as_str()),
        ("提取模型", draft.extraction_model.as_str()),
    ] {
        validate_model_id(label, model)?;
    }
    if let Some(model) = draft.fallback_model.as_deref() {
        validate_model_id("备用模型", model)?;
    }
    if draft.api_base_url.is_empty() {
        return Err("API基础地址不能为空".to_string());
    }
    if draft.api_endpoint.is_empty() {
        draft.api_endpoint = format!(
            "{}{}",
            draft.api_base_url.trim_end_matches('/'),
            draft.api_protocol.endpoint_suffix()
        );
    }
    if !(10..=900).contains(&draft.timeout_seconds) {
        return Err("超时必须位于10至900秒".to_string());
    }
    if draft.max_retries > 10 {
        return Err("最大重试不能超过10次".to_string());
    }
    if !(1..=16).contains(&draft.max_concurrency) {
        return Err("最大并发必须位于1至16".to_string());
    }
    if !(1..=100_000).contains(&draft.max_output_tokens) {
        return Err("输出Token上限必须位于1至100000".to_string());
    }
    if !(1..=100).contains(&draft.max_tool_calls) {
        return Err("工具调用上限必须位于1至100".to_string());
    }
    Ok(())
}

fn validate_model_id(label: &str, value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value.len() <= 120
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'));
    if valid {
        Ok(())
    } else {
        Err(format!("{label}ID格式无效"))
    }
}

fn connection_settings_changed(existing: &StoredOpenAiProfile, draft: &OpenAiProfileDraft) -> bool {
    existing.api_base_url != draft.api_base_url
        || existing.api_protocol != draft.api_protocol
        || effective_endpoint(existing) != draft.api_endpoint
        || existing.token_limit_field != draft.token_limit_field
        || existing.api_workspace_web_search_mode != draft.api_workspace_web_search_mode
        || existing.api_example_template != draft.api_example_template
        || existing.research_model != draft.research_model
        || existing.extraction_model != draft.extraction_model
        || existing.fallback_model != draft.fallback_model
        || existing.reasoning_effort != draft.reasoning_effort
        || existing.timeout_seconds != draft.timeout_seconds
        || existing.max_retries != draft.max_retries
        || existing.max_concurrency != draft.max_concurrency
        || existing.max_output_tokens != draft.max_output_tokens
        || existing.max_tool_calls != draft.max_tool_calls
        || existing.search_context_size != draft.search_context_size
}

fn validate_profile(profile: &StoredOpenAiProfile) -> Result<(), String> {
    gateway_config(profile).map(|_| ())
}

fn gateway_config(profile: &StoredOpenAiProfile) -> Result<GatewayConfig, String> {
    let mut config: GatewayConfig = serde_json::from_str(DEFAULT_CONFIG)
        .map_err(|error| format!("内置兼容 API配置损坏：{error}"))?;
    config.api_base_url = profile.api_base_url.clone();
    config.api_protocol = profile.api_protocol;
    config.request_endpoint = Some(effective_endpoint(profile));
    config.token_limit_field = profile.token_limit_field;
    config.api_workspace_web_search_mode = profile.api_workspace_web_search_mode;
    config.research_model = profile.research_model.clone();
    config.extraction_model = profile.extraction_model.clone();
    config.fallback_model = profile.fallback_model.clone();
    config.reasoning_effort = profile.reasoning_effort;
    config.timeout_seconds = profile.timeout_seconds;
    config.max_retries = profile.max_retries;
    config.max_concurrency = profile.max_concurrency;
    config.max_output_tokens = profile.max_output_tokens;
    config.max_tool_calls = profile.max_tool_calls;
    config.search_context_size = profile.search_context_size;
    config.credentials.mode = CredentialMode::WindowsCredentialManager;
    config.credentials.credential_target = credential_target(profile.id);
    config.credentials.environment_variable = "OPENAI_API_KEY".to_string();
    config.credentials.deployment_mode = "local_desktop".to_string();
    let configured_models = [
        Some(config.research_model.as_str()),
        Some(config.extraction_model.as_str()),
        config.fallback_model.as_deref(),
    ];
    if configured_models
        .into_iter()
        .flatten()
        .any(|model| !config.budget.model_pricing.contains_key(model))
    {
        config.budget.daily_budget_usd = None;
        config.budget.monthly_budget_usd = None;
        config.budget.per_request_max_usd = None;
        config.budget.model_pricing.clear();
    }
    config.validate().map_err(|error| error.to_string())?;
    Ok(config)
}

fn effective_endpoint(profile: &StoredOpenAiProfile) -> String {
    let endpoint = profile.api_endpoint.trim();
    if endpoint.is_empty() {
        format!(
            "{}{}",
            profile.api_base_url.trim_end_matches('/'),
            profile.api_protocol.endpoint_suffix()
        )
    } else {
        endpoint.trim_end_matches('/').to_string()
    }
}

fn credential_target(profile_id: Uuid) -> String {
    format!("{CREDENTIAL_PREFIX}/{profile_id}")
}

fn parse_profile_id(value: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|error| format!("兼容 API配置ID无效：{error}"))
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_metadata_never_serializes_api_key() {
        let directory = tempfile::tempdir().expect("临时目录");
        let store = OpenAiProfileStore::new(directory.path().join("profiles.json"));
        let state = store.list().expect("读取配置");
        let profile = state.profiles.first().expect("默认配置");
        let draft = OpenAiProfileDraft {
            id: Some(profile.id.clone()),
            name: "测试配置".to_string(),
            api_key: None,
            api_base_url: profile.api_base_url.clone(),
            api_protocol: profile.api_protocol,
            api_endpoint: profile.api_endpoint.clone(),
            token_limit_field: profile.token_limit_field,
            api_workspace_web_search_mode: profile.api_workspace_web_search_mode,
            api_example_template: profile.api_example_template.clone(),
            research_model: profile.research_model.clone(),
            extraction_model: profile.extraction_model.clone(),
            fallback_model: None,
            reasoning_effort: ReasoningEffort::Medium,
            timeout_seconds: 180,
            max_retries: 3,
            max_concurrency: 2,
            max_output_tokens: 12_000,
            max_tool_calls: 12,
            search_context_size: SearchContextSize::High,
        };
        store.save(draft).expect("保存配置");
        let raw = fs::read_to_string(directory.path().join("profiles.json")).expect("读取文件");
        assert!(!raw.contains("api_key"));
        assert!(!raw.contains("Bearer"));
    }

    #[test]
    fn profile_names_are_unique_case_insensitively() {
        let directory = tempfile::tempdir().expect("临时目录");
        let store = OpenAiProfileStore::new(directory.path().join("profiles.json"));
        let mut initial = store.list().expect("读取配置");
        let base = initial.profiles.remove(0);
        let draft = |name: &str| OpenAiProfileDraft {
            id: None,
            name: name.to_string(),
            api_key: None,
            api_base_url: base.api_base_url.clone(),
            api_protocol: base.api_protocol,
            api_endpoint: base.api_endpoint.clone(),
            token_limit_field: base.token_limit_field,
            api_workspace_web_search_mode: base.api_workspace_web_search_mode,
            api_example_template: base.api_example_template.clone(),
            research_model: base.research_model.clone(),
            extraction_model: base.extraction_model.clone(),
            fallback_model: None,
            reasoning_effort: ReasoningEffort::Medium,
            timeout_seconds: 180,
            max_retries: 3,
            max_concurrency: 2,
            max_output_tokens: 12_000,
            max_tool_calls: 12,
            search_context_size: SearchContextSize::High,
        };
        store.save(draft("Second")).expect("首次保存");
        assert!(store.save(draft("second")).is_err());
    }

    #[test]
    fn changing_connection_settings_resets_previous_success() {
        let directory = tempfile::tempdir().expect("临时目录");
        let store = OpenAiProfileStore::new(directory.path().join("profiles.json"));
        let profile_id = {
            let mut state = store.lock().expect("配置锁");
            let profile = state.profiles.first_mut().expect("默认配置");
            profile.last_test_status = "success".to_string();
            profile.last_test_message = Some("旧连接测试".to_string());
            profile.last_tested_at = Some(Utc::now());
            let profile_id = profile.id;
            persist_state(&store.path, &state).expect("保存测试状态");
            profile_id
        };
        let current = store.list().expect("读取配置").profiles.remove(0);
        let saved = store
            .save(OpenAiProfileDraft {
                id: Some(profile_id.to_string()),
                name: current.name,
                api_key: None,
                api_base_url: current.api_base_url,
                api_protocol: current.api_protocol,
                api_endpoint: current.api_endpoint,
                token_limit_field: current.token_limit_field,
                api_workspace_web_search_mode: current.api_workspace_web_search_mode,
                api_example_template: current.api_example_template,
                research_model: "gpt-5.5-updated".to_string(),
                extraction_model: current.extraction_model,
                fallback_model: current.fallback_model,
                reasoning_effort: current.reasoning_effort,
                timeout_seconds: current.timeout_seconds,
                max_retries: current.max_retries,
                max_concurrency: current.max_concurrency,
                max_output_tokens: current.max_output_tokens,
                max_tool_calls: current.max_tool_calls,
                search_context_size: current.search_context_size,
            })
            .expect("保存修改");
        assert_eq!(saved.last_test_status, "untested");
        assert!(saved.last_test_message.is_none());
        assert!(saved.last_tested_at.is_none());
    }

    #[test]
    fn api_example_extracts_key_and_persists_only_redacted_template() {
        let secret = "sk-test-profile-secret-1234567890";
        let mut draft = OpenAiProfileDraft {
            id: None,
            name: "Example profile".to_string(),
            api_key: None,
            api_base_url: "https://api.openai.com/v1".to_string(),
            api_protocol: ApiProtocol::Responses,
            api_endpoint: "https://api.openai.com/v1/responses".to_string(),
            token_limit_field: TokenLimitField::MaxOutputTokens,
            api_workspace_web_search_mode: ApiWorkspaceWebSearchMode::Auto,
            api_example_template: Some(format!(
                "curl https://api.gptsapi.net/v1/responses -H \"Authorization: Bearer {secret}\" -d '{{\"model\":\"gpt-5.6-sol\",\"input\":\"hi\",\"max_tokens\":1000}}'"
            )),
            research_model: "gpt-5.6-sol".to_string(),
            extraction_model: "gpt-5.6-sol".to_string(),
            fallback_model: None,
            reasoning_effort: ReasoningEffort::Medium,
            timeout_seconds: 180,
            max_retries: 3,
            max_concurrency: 2,
            max_output_tokens: 1_000,
            max_tool_calls: 12,
            search_context_size: SearchContextSize::High,
        };
        normalize_draft(&mut draft).expect("解析并脱敏API Example");
        assert_eq!(draft.api_key.as_deref(), Some(secret));
        let stored = draft.api_example_template.expect("脱敏示例");
        assert!(!stored.contains(secret));
        assert!(stored.contains("YOUR_API_KEY"));
    }

    #[test]
    fn profile_limits_are_enforced_by_rust() {
        let mut draft = OpenAiProfileDraft {
            id: None,
            name: "Limit test".to_string(),
            api_key: None,
            api_base_url: "https://api.openai.com/v1".to_string(),
            api_protocol: ApiProtocol::Responses,
            api_endpoint: "https://api.openai.com/v1/responses".to_string(),
            token_limit_field: TokenLimitField::MaxOutputTokens,
            api_workspace_web_search_mode: ApiWorkspaceWebSearchMode::Auto,
            api_example_template: None,
            research_model: "gpt-5.5".to_string(),
            extraction_model: "gpt-5.5".to_string(),
            fallback_model: None,
            reasoning_effort: ReasoningEffort::Medium,
            timeout_seconds: 9,
            max_retries: 3,
            max_concurrency: 2,
            max_output_tokens: 12_000,
            max_tool_calls: 12,
            search_context_size: SearchContextSize::High,
        };
        assert!(normalize_draft(&mut draft).is_err());
    }

    #[test]
    fn legacy_profile_without_protocol_keeps_responses_defaults() {
        let directory = tempfile::tempdir().expect("临时目录");
        let path = directory.path().join("profiles.json");
        let profile_id = Uuid::new_v4();
        let now = Utc::now();
        let legacy = serde_json::json!({
            "version": STORE_VERSION,
            "active_profile_id": profile_id,
            "profiles": [{
                "id": profile_id,
                "name": "Legacy",
                "api_base_url": "https://api.openai.com/v1",
                "research_model": "gpt-5.5",
                "extraction_model": "gpt-5.5",
                "fallback_model": null,
                "reasoning_effort": "medium",
                "timeout_seconds": 180,
                "max_retries": 3,
                "max_concurrency": 2,
                "max_output_tokens": 12000,
                "max_tool_calls": 12,
                "search_context_size": "high",
                "last_test_status": "untested",
                "last_test_message": null,
                "last_tested_at": null,
                "created_at": now,
                "updated_at": now
            }]
        });
        fs::write(&path, serde_json::to_vec_pretty(&legacy).expect("序列化")).expect("写入");
        let store = OpenAiProfileStore::new(path);
        let profile = store.list().expect("读取").profiles.remove(0);
        assert_eq!(profile.api_protocol, ApiProtocol::Responses);
        assert_eq!(profile.api_endpoint, "https://api.openai.com/v1/responses");
    }
}
