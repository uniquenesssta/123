use super::{
    artifacts::{built_in_research_prompt, built_in_research_schema},
    attempt_audit::PortAttemptSink,
    gateway::built_in_gateway,
    references::{citation_drafts, source_drafts},
    validation::validate_command,
    *,
};

pub(crate) async fn execute(
    artifacts: &dyn ResearchArtifactPort,
    audit: &dyn ResearchGatewayAuditPort,
    pipeline: &dyn FactPipelineAccess,
    command: OpenAiResearchCommand,
    cancellation: CancellationToken,
) -> ApplicationResult<GatewayExecution> {
    validate_command(&command)?;
    let usage = audit.usage_totals().await?;
    let attempt_number_offset = audit.attempt_number_offset(command.research_run_id).await?;
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
    artifacts
        .record_run_event(&ResearchRunEventDraft {
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

    let sink = PortAttemptSink {
        port: audit,
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
            if let Err(error) = audit
                .append_web_references(command.research_run_id, &sources, &citations)
                .await
            {
                let gateway_error = GatewayError::new(
                    GatewayErrorCategory::Persistence,
                    format!("OpenAI引用持久化失败：{error}"),
                    true,
                    "恢复PostgreSQL连接后按原研究任务幂等键重试；不要重新创建比赛或快照",
                );
                record_gateway_failure(
                    artifacts,
                    &command,
                    &gateway_error,
                    Some(&execution.response.response_id),
                    attempt_number_offset,
                )
                .await?;
                return Err(ApplicationError::ResearchGateway(gateway_error));
            }
            let pipeline_summary = match fact_pipeline::process_p4_research_evidence(
                pipeline,
                ProcessResearchEvidenceCommand {
                    research_run_id: command.research_run_id,
                    response_id: execution.response.response_id.clone(),
                    retrieved_at,
                    output: execution.response.output.clone(),
                    citations: execution.response.citations.clone(),
                    sources: execution.response.sources.clone(),
                },
            )
            .await
            {
                Ok(summary) => summary,
                Err(error) => {
                    artifacts
                        .record_run_event(&ResearchRunEventDraft {
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
            artifacts
                .record_run_event(&ResearchRunEventDraft {
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
            record_gateway_failure(artifacts, &command, &error, None, attempt_number_offset)
                .await?;
            Err(ApplicationError::ResearchGateway(error))
        }
    }
}

async fn record_gateway_failure(
    artifacts: &dyn ResearchArtifactPort,
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
    artifacts
        .record_run_event(&ResearchRunEventDraft {
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
