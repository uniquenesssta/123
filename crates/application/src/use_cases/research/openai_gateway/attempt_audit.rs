use super::*;

pub(super) struct PortAttemptSink<'a> {
    pub(super) port: &'a dyn ResearchGatewayAuditPort,
    pub(super) research_run_id: Uuid,
}

#[async_trait]
impl GatewayAttemptSink for PortAttemptSink<'_> {
    async fn record(&self, attempt: &GatewayAttempt) -> Result<(), GatewayError> {
        let error_category = attempt
            .error
            .as_ref()
            .map(|error| error.category.as_str().to_string());
        let error_message = attempt
            .error
            .as_ref()
            .map(|error| error.user_message.clone());
        self.port
            .append_attempt(&OpenAiAttemptDraft {
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

pub(super) fn normalize_attempt_status(status: &str) -> String {
    match status {
        "queued" | "in_progress" | "completed" | "failed" | "cancelled" | "incomplete" => {
            status.to_string()
        }
        "succeeded" => "completed".to_string(),
        _ => "failed".to_string(),
    }
}
