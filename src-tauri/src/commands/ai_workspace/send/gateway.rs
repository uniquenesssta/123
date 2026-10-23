use super::{
    content::plain_text_system_instructions, request::SendApiWorkspaceCommand,
    validation::ValidatedMessage,
};
use crate::{bootstrap::AppState, openai_profiles::OpenAiProfileSummary};
use football_application::AiWorkspaceRequestLease;
use football_research_gateway::{
    GatewayOperation, OpenAiResearchGateway, PlainTextGatewayExecution, PlainTextGatewayRequest,
    PlainTextMessage,
};
use serde_json::json;
use uuid::Uuid;

pub(super) async fn execute(
    state: &AppState,
    command: &SendApiWorkspaceCommand,
    input: &ValidatedMessage<'_>,
    connection: &(OpenAiProfileSummary, OpenAiResearchGateway),
    session_id: Uuid,
    messages: Vec<PlainTextMessage>,
    registration: AiWorkspaceRequestLease,
) -> Result<PlainTextGatewayExecution, String> {
    let (profile, gateway) = connection;
    let request_id = input.request_id;
    let message = input.message;
    let preset_spec = &input.preset_spec;
    let usage = state
        .service
        .api_workspace_openai_usage_totals()
        .await
        .map_err(|error| error.to_string())?;
    let trace_id = format!("ai-chat:{session_id}:{request_id}");
    let endpoint = gateway.config().request_endpoint();
    let protocol = serde_json::to_value(profile.api_protocol)
        .ok()
        .and_then(|value| value.as_str().map(ToString::to_string))
        .unwrap_or_else(|| "responses".to_string());
    let _ = state.runtime_log.record(
        "info",
        "ai_chat",
        "request_started",
        Some(&trace_id),
        json!({
            "request_id": request_id,
            "session_id": session_id,
            "profile_id": profile.id,
            "protocol": protocol,
            "endpoint": endpoint,
            "message_chars": message.chars().count(),
            "history_messages": messages.len(),
            "context_attached": command.include_context,
        }),
    );
    let request = PlainTextGatewayRequest {
        operation: GatewayOperation::Extraction,
        trace_id: trace_id.clone(),
        static_instructions: plain_text_system_instructions(
            &preset_spec.preset.title,
            &preset_spec.instructions,
        ),
        messages,
        daily_spend_usd: usage.today_cost_usd,
        monthly_spend_usd: usage.month_cost_usd,
        attempt_number_offset: 0,
    };
    let execution_result = gateway
        .execute_plain_text(&request, registration.cancellation())
        .await;
    drop(registration);
    let execution = match execution_result {
        Ok(execution) => execution,
        Err(error) => {
            let _ = state.runtime_log.record(
                if error.category == football_research_gateway::GatewayErrorCategory::Cancelled {
                    "info"
                } else {
                    "error"
                },
                "ai_chat",
                if error.category == football_research_gateway::GatewayErrorCategory::Cancelled {
                    "request_cancelled"
                } else {
                    "request_failed"
                },
                Some(&trace_id),
                json!({
                    "request_id": request_id,
                    "session_id": session_id,
                    "protocol": protocol,
                    "endpoint": endpoint,
                    "error_category": error.category.as_str(),
                    "provider_status": error.provider_status,
                    "error": error.to_string(),
                }),
            );
            return Err(error.to_string());
        }
    };
    let latency_ms = execution
        .attempts
        .last()
        .map_or(0, |attempt| attempt.latency_ms);
    let _ = state.runtime_log.record(
        "info",
        "ai_chat",
        "request_completed",
        Some(&trace_id),
        json!({
            "request_id": request_id,
            "session_id": session_id,
            "protocol": protocol,
            "endpoint": endpoint,
            "status": execution.response.status,
            "provider_status": execution.attempts.last().and_then(|attempt| attempt.provider_status),
            "latency_ms": latency_ms,
            "attempt_count": execution.attempts.len(),
            "response_chars": execution.response.text.chars().count(),
        }),
    );
    Ok(execution)
}
