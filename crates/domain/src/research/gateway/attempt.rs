use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiAttemptDraft {
    pub research_run_id: Uuid,
    pub attempt_number: u32,
    pub model_id: String,
    pub request_fingerprint: String,
    pub request_payload: Value,
    #[serde(default)]
    pub response_id: Option<String>,
    #[serde(default)]
    pub provider_request_id: Option<String>,
    #[serde(default)]
    pub provider_status: Option<u16>,
    pub status: String,
    #[serde(default)]
    pub token_usage: Value,
    pub latency_ms: u64,
    pub search_call_count: u32,
    #[serde(default)]
    pub estimated_cost_usd: Option<f64>,
    #[serde(default)]
    pub raw_response: Option<Value>,
    #[serde(default)]
    pub error_category: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    pub retryable: bool,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiAttemptRecord {
    pub id: Uuid,
    pub research_run_id: Uuid,
    pub attempt_number: u32,
    pub attempt_fingerprint: String,
    pub response_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
