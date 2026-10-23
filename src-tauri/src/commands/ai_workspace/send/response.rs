use football_domain::ApiWorkspaceMessageDraft;
use football_research_gateway::PlainTextGatewayExecution;
use serde_json::json;
use uuid::Uuid;

pub(super) fn assistant_message(
    session_id: Uuid,
    execution: PlainTextGatewayExecution,
) -> ApiWorkspaceMessageDraft {
    let token_usage = json!({
        "input_tokens": execution.response.usage.input_tokens,
        "cached_input_tokens": execution.response.usage.cached_input_tokens,
        "output_tokens": execution.response.usage.output_tokens,
        "total_tokens": execution.response.usage.total_tokens,
        "estimated_cost_usd": execution
            .attempts
            .last()
            .and_then(|attempt| attempt.estimated_cost_usd),
    });
    ApiWorkspaceMessageDraft {
        session_id,
        role: "assistant".to_string(),
        content: execution.response.text,
        structured_payload: json!({"mode": "plain_text", "legacy": false}),
        citations: json!([]),
        attachments: json!([]),
        provider_response_id: Some(execution.response.response_id),
        model_id: Some(execution.response.model_id),
        token_usage,
    }
}
