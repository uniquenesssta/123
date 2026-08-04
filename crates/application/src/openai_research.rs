use super::{ApplicationError, ApplicationResult, ApplicationService};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{
    OpenAiAttemptDraft, PromptVersionDraft, ResearchRunEventDraft, ResearchRunStatus,
    SchemaVersionDraft, WebCitationDraft, WebSourceDraft, P4_RESEARCH_OUTPUT_SCHEMA_VERSION,
    P4_RESEARCH_PROMPT_VERSION,
};
use football_research_gateway::{
    CancellationToken, DefaultApiKeyProvider, GatewayAttempt, GatewayAttemptSink, GatewayConfig,
    GatewayError, GatewayErrorCategory, GatewayExecution, GatewayOperation, GatewayRequest,
    OpenAiResearchGateway, ReqwestTransport, WebCitation, WebSource,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

const RESEARCH_SCHEMA_KEY: &str = "p4-openai-research-output";
const RESEARCH_PROMPT_KEY: &str = "p4-openai-research-system";
const RESEARCH_SCHEMA_NAME: &str = "p4_openai_research_output";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiResearchCommand {
    pub research_run_id: Uuid,
    pub trace_id: Uuid,
    pub match_key: String,
    pub data_cutoff_at: DateTime<Utc>,
    #[serde(default = "default_operation")]
    pub operation: GatewayOperation,
    pub dynamic_context: Value,
    pub requested_fact_keys: Vec<String>,
}

fn default_operation() -> GatewayOperation {
    GatewayOperation::Research
}

impl ApplicationService {
    pub(super) async fn register_openai_research_artifacts(
        &self,
        store: &football_persistence_postgres::PostgresStore,
    ) -> ApplicationResult<()> {
        store
            .register_schema_version(&built_in_research_schema())
            .await?;
        store
            .register_prompt_version(&built_in_research_prompt())
            .await?;
        self.register_fact_pipeline_artifacts(store).await?;
        Ok(())
    }

    pub async fn execute_p4_openai_research(
        &self,
        command: OpenAiResearchCommand,
        cancellation: CancellationToken,
    ) -> ApplicationResult<GatewayExecution> {
        validate_command(&command)?;
        let store = self.active_store().await?;
        let usage = store.openai_usage_totals().await?;
        let attempt_number_offset = store
            .openai_attempt_number_offset(command.research_run_id)
            .await?;
        let gateway = built_in_gateway()?;
        let schema = built_in_research_schema();
        let prompt = built_in_research_prompt();
        let request = GatewayRequest {
            operation: command.operation,
            trace_id: command.trace_id.to_string(),
            match_key: command.match_key.clone(),
            data_cutoff_at: command.data_cutoff_at,
            schema_name: RESEARCH_SCHEMA_NAME.to_string(),
            schema_version: P4_RESEARCH_OUTPUT_SCHEMA_VERSION.to_string(),
            schema: schema.schema_body,
            static_instructions: prompt.content,
            dynamic_context: command.dynamic_context.clone(),
            requested_fact_keys: command.requested_fact_keys.clone(),
            daily_spend_usd: usage.today_cost_usd,
            monthly_spend_usd: usage.month_cost_usd,
            attempt_number_offset,
        };
        self.record_p4_research_run_event(ResearchRunEventDraft {
            research_run_id: command.research_run_id,
            idempotency_key: format!(
                "openai:{}:running:{}",
                command.trace_id, attempt_number_offset
            ),
            status: ResearchRunStatus::Running,
            response_id: None,
            model_id: None,
            token_usage: json!({}),
            error_category: None,
            error_message: None,
            payload: json!({
                "gateway_contract": "football.p4-research-gateway.v1",
                "operation": command.operation,
                "schema_version": P4_RESEARCH_OUTPUT_SCHEMA_VERSION,
                "prompt_version": P4_RESEARCH_PROMPT_VERSION,
                "attempt_number_offset": attempt_number_offset
            }),
        })
        .await?;

        let sink = PersistenceAttemptSink {
            store: store.clone(),
            research_run_id: command.research_run_id,
        };
        match gateway
            .execute_with_sink(&request, &cancellation, &sink)
            .await
        {
            Ok(execution) => {
                let retrieved_at = Utc::now();
                let citations = citation_drafts(
                    command.research_run_id,
                    &execution.response.response_id,
                    &execution.response.citations,
                    retrieved_at,
                )?;
                let sources = source_drafts(
                    command.research_run_id,
                    &execution.response.response_id,
                    &execution.response.sources,
                    retrieved_at,
                )?;
                if let Err(error) = store.append_web_references(&citations, &sources).await {
                    let gateway_error = GatewayError::new(
                        GatewayErrorCategory::Persistence,
                        format!("OpenAI引用持久化失败：{error}"),
                        true,
                        "恢复PostgreSQL连接后按原研究任务幂等键重试；不要重新创建比赛或快照",
                    );
                    self.record_gateway_failure(
                        &command,
                        &gateway_error,
                        Some(&execution.response.response_id),
                        attempt_number_offset,
                    )
                    .await?;
                    return Err(ApplicationError::ResearchGateway(gateway_error));
                }
                let pipeline_summary = match self
                    .process_p4_research_evidence(super::ProcessResearchEvidenceCommand {
                        research_run_id: command.research_run_id,
                        response_id: execution.response.response_id.clone(),
                        retrieved_at,
                        output: execution.response.output.clone(),
                        citations: execution.response.citations.clone(),
                        sources: execution.response.sources.clone(),
                    })
                    .await
                {
                    Ok(summary) => summary,
                    Err(error) => {
                        self.record_p4_research_run_event(ResearchRunEventDraft {
                            research_run_id: command.research_run_id,
                            idempotency_key: format!(
                                "pipeline:{}:{}:failed",
                                command.trace_id, execution.response.response_id
                            ),
                            status: ResearchRunStatus::Partial,
                            response_id: Some(execution.response.response_id.clone()),
                            model_id: Some(execution.response.model_id.clone()),
                            token_usage: serde_json::to_value(&execution.response.usage)?,
                            error_category: Some("fact_pipeline".to_string()),
                            error_message: Some(error.to_string()),
                            payload: json!({
                                "retryable": true,
                                "recommended_action": "修复实体目录、时间字段、来源策略或数据库连接后，按原研究任务幂等键重新执行证据流水线",
                                "strict_schema_validated": true,
                                "rust_validated": true,
                                "references_persisted": true
                            }),
                        })
                        .await?;
                        return Err(ApplicationError::Validation(format!(
                            "联网结果已保存，但实体、时间、冲突或证据路由失败：{error}"
                        )));
                    }
                };
                let run_status = if pipeline_summary.has_blockers() {
                    ResearchRunStatus::Partial
                } else {
                    ResearchRunStatus::Succeeded
                };
                self.record_p4_research_run_event(ResearchRunEventDraft {
                    research_run_id: command.research_run_id,
                    idempotency_key: format!(
                        "openai:{}:{}:{}",
                        command.trace_id,
                        execution.response.response_id,
                        run_status.as_str()
                    ),
                    status: run_status,
                    response_id: Some(execution.response.response_id.clone()),
                    model_id: Some(execution.response.model_id.clone()),
                    token_usage: serde_json::to_value(&execution.response.usage)?,
                    error_category: None,
                    error_message: None,
                    payload: json!({
                        "provider_request_id": execution.response.provider_request_id.clone(),
                        "citation_count": citations.len(),
                        "source_count": sources.len(),
                        "search_call_count": execution.response.search_call_count,
                        "strict_schema_validated": true,
                        "rust_validated": true,
                        "fact_pipeline": pipeline_summary,
                        "ready_for_stage_f": !pipeline_summary.has_blockers()
                    }),
                })
                .await?;
                Ok(execution)
            }
            Err(error) => {
                self.record_gateway_failure(&command, &error, None, attempt_number_offset)
                    .await?;
                Err(ApplicationError::ResearchGateway(error))
            }
        }
    }

    async fn record_gateway_failure(
        &self,
        command: &OpenAiResearchCommand,
        error: &GatewayError,
        response_id: Option<&str>,
        attempt_number_offset: u32,
    ) -> ApplicationResult<()> {
        let status = if error.category == GatewayErrorCategory::Cancelled {
            ResearchRunStatus::Cancelled
        } else {
            ResearchRunStatus::Failed
        };
        self.record_p4_research_run_event(ResearchRunEventDraft {
            research_run_id: command.research_run_id,
            idempotency_key: format!(
                "openai:{}:{}:{}:{}:{}",
                command.trace_id,
                status.as_str(),
                error.category.as_str(),
                response_id.unwrap_or("none"),
                attempt_number_offset
            ),
            status,
            response_id: response_id.map(ToString::to_string),
            model_id: None,
            token_usage: json!({}),
            error_category: Some(error.category.as_str().to_string()),
            error_message: Some(error.user_message.clone()),
            payload: json!({
                "retryable": error.recovery.retryable,
                "recommended_action": error.recovery.action.clone(),
                "provider_status": error.provider_status,
                "provider_code": error.provider_code.clone()
            }),
        })
        .await?;
        Ok(())
    }
}

fn built_in_gateway() -> Result<OpenAiResearchGateway, GatewayError> {
    OpenAiResearchGateway::new(
        built_in_gateway_config()?,
        Arc::new(ReqwestTransport::new()?),
        Arc::new(DefaultApiKeyProvider),
    )
}

fn citation_drafts(
    research_run_id: Uuid,
    response_id: &str,
    citations: &[WebCitation],
    retrieved_at: DateTime<Utc>,
) -> ApplicationResult<Vec<WebCitationDraft>> {
    citations
        .iter()
        .map(|citation| {
            Ok(WebCitationDraft {
                research_run_id,
                response_id: response_id.to_string(),
                url: citation.url.clone(),
                title: citation.title.clone(),
                domain: verified_reference_domain(&citation.url)?,
                output_index: u32::try_from(citation.location.output_index).map_err(|_| {
                    ApplicationError::Validation(
                        "OpenAI引用输出位置超出u32范围，已拒绝持久化".to_string(),
                    )
                })?,
                start_index: optional_u32(citation.location.start_index, "引用起始位置")?,
                end_index: optional_u32(citation.location.end_index, "引用结束位置")?,
                retrieved_at,
            })
        })
        .collect()
}

fn source_drafts(
    research_run_id: Uuid,
    response_id: &str,
    sources: &[WebSource],
    retrieved_at: DateTime<Utc>,
) -> ApplicationResult<Vec<WebSourceDraft>> {
    sources
        .iter()
        .map(|source| {
            Ok(WebSourceDraft {
                research_run_id,
                response_id: response_id.to_string(),
                url: source.url.clone(),
                title: source.title.clone(),
                domain: verified_reference_domain(&source.url)?,
                retrieved_at,
            })
        })
        .collect()
}

fn verified_reference_domain(value: &str) -> ApplicationResult<String> {
    let url = url::Url::parse(value).map_err(|_| {
        ApplicationError::Validation(format!("OpenAI来源URL无效，无法保存真实域名：{value}"))
    })?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return Err(ApplicationError::Validation(format!(
            "OpenAI来源URL必须使用HTTPS且不能包含用户名或密码：{value}"
        )));
    }
    url.host_str()
        .map(|host| host.trim_start_matches("www.").to_lowercase())
        .filter(|host| !host.is_empty())
        .ok_or_else(|| ApplicationError::Validation(format!("OpenAI来源URL缺少域名：{value}")))
}

fn optional_u32(value: Option<usize>, label: &str) -> ApplicationResult<Option<u32>> {
    value
        .map(|value| {
            u32::try_from(value).map_err(|_| {
                ApplicationError::Validation(format!("{label}超出u32范围，已拒绝持久化"))
            })
        })
        .transpose()
}

struct PersistenceAttemptSink {
    store: football_persistence_postgres::PostgresStore,
    research_run_id: Uuid,
}

#[async_trait]
impl GatewayAttemptSink for PersistenceAttemptSink {
    async fn record(&self, attempt: &GatewayAttempt) -> Result<(), GatewayError> {
        let error_category = attempt
            .error
            .as_ref()
            .map(|error| error.category.as_str().to_string());
        let error_message = attempt
            .error
            .as_ref()
            .map(|error| error.user_message.clone());
        self.store
            .append_openai_attempt(&OpenAiAttemptDraft {
                research_run_id: self.research_run_id,
                attempt_number: attempt.attempt_number,
                model_id: attempt.model_id.clone(),
                request_fingerprint: attempt.request_fingerprint.clone(),
                request_payload: attempt.request_payload.clone(),
                response_id: attempt.response_id.clone(),
                provider_request_id: attempt.provider_request_id.clone(),
                provider_status: attempt.provider_status,
                status: normalize_attempt_status(&attempt.status),
                token_usage: serde_json::to_value(&attempt.usage).unwrap_or_else(|_| json!({})),
                latency_ms: attempt.latency_ms,
                search_call_count: attempt.search_call_count,
                estimated_cost_usd: attempt.estimated_cost_usd,
                raw_response: attempt.raw_response.clone(),
                error_category,
                error_message,
                retryable: attempt
                    .error
                    .as_ref()
                    .is_some_and(|error| error.recovery.retryable),
                started_at: attempt.started_at,
                finished_at: attempt.finished_at,
            })
            .await
            .map(|_| ())
            .map_err(|error| {
                GatewayError::new(
                    GatewayErrorCategory::Persistence,
                    format!("OpenAI尝试审计写入失败：{error}"),
                    true,
                    "恢复PostgreSQL连接后按原研究任务幂等键重试",
                )
            })
    }
}

fn normalize_attempt_status(status: &str) -> String {
    match status {
        "queued" | "in_progress" | "completed" | "failed" | "cancelled" | "incomplete" => {
            status.to_string()
        }
        "succeeded" => "completed".to_string(),
        _ => "failed".to_string(),
    }
}

fn built_in_gateway_config() -> Result<GatewayConfig, GatewayError> {
    let config: GatewayConfig = serde_json::from_str(include_str!(
        "../../../src-tauri/resources/research/openai_gateway.json"
    ))
    .map_err(|error| {
        GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            format!("内置OpenAI研究网关配置无效：{error}"),
            false,
            "修正版本化网关配置后重新构建应用",
        )
    })?;
    config.validate()?;
    Ok(config)
}

fn built_in_research_schema() -> SchemaVersionDraft {
    SchemaVersionDraft {
        schema_key: RESEARCH_SCHEMA_KEY.to_string(),
        version: "2.0.0".to_string(),
        schema_kind: "openai_structured_output".to_string(),
        schema_body: serde_json::from_str(include_str!(
            "../../../schemas/research-output.schema.json"
        ))
        .expect("内置公开研究输出Schema必须有效"),
        description: Some("OpenAI Web Search事实研究严格输出契约".to_string()),
        metadata: json!({
            "schema_id": P4_RESEARCH_OUTPUT_SCHEMA_VERSION,
            "stage": "E",
            "strict": true,
            "rust_second_validation": true,
            "atomic_claims": true,
            "entity_time_conflict_pipeline": true
        }),
    }
}

fn built_in_research_prompt() -> PromptVersionDraft {
    PromptVersionDraft {
        prompt_key: RESEARCH_PROMPT_KEY.to_string(),
        version: P4_RESEARCH_PROMPT_VERSION.to_string(),
        prompt_role: "research_system".to_string(),
        content: include_str!("../../../src-tauri/resources/research/public_research_prompt.txt")
            .to_string(),
        metadata: json!({
            "stage": "E",
            "static_prefix": true,
            "prediction_prohibited": true,
            "atomic_source_backed_claims": true,
            "entity_resolution_stage": "E"
        }),
    }
}

fn validate_command(command: &OpenAiResearchCommand) -> ApplicationResult<()> {
    if command.match_key.trim().is_empty()
        || command.match_key.chars().count() > 200
        || command.requested_fact_keys.is_empty()
        || command.requested_fact_keys.len() > 31
    {
        return Err(ApplicationError::Validation(
            "OpenAI研究任务必须包含有效比赛键和1至31个事实字段".to_string(),
        ));
    }
    if command
        .requested_fact_keys
        .iter()
        .any(|field| field.trim().is_empty() || field.chars().count() > 100)
    {
        return Err(ApplicationError::Validation(
            "OpenAI研究任务包含空字段或超过100字符的字段".to_string(),
        ));
    }
    let unique_fields: std::collections::BTreeSet<_> = command
        .requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect();
    if unique_fields.len() != command.requested_fact_keys.len() {
        return Err(ApplicationError::Validation(
            "OpenAI研究任务不能包含重复事实字段".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_gateway_is_strict_and_has_no_secret() {
        let config = built_in_gateway_config().expect("config");
        assert!(!config.research_model.is_empty());
        assert!(!config.extraction_model.is_empty());
        assert!(!config.store);
        let serialized = include_str!("../../../src-tauri/resources/research/openai_gateway.json");
        assert!(!serialized.contains("sk-"));
        assert!(!serialized.contains("api_key"));
        let schema = built_in_research_schema();
        assert_eq!(schema.schema_body["additionalProperties"], false);
        assert!(built_in_research_prompt()
            .content
            .contains("do not calculate probabilities"));
    }
}
