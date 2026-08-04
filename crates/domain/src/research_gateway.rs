use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const P4_RESEARCH_GATEWAY_CONTRACT_VERSION: &str = "football.p4-research-gateway.v1";
pub const P4_RESEARCH_OUTPUT_SCHEMA_VERSION: &str = "football.p4-research-output.v2";
pub const P4_RESEARCH_PROMPT_VERSION: &str = "2.0.0";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebCitationDraft {
    pub research_run_id: Uuid,
    pub response_id: String,
    pub url: String,
    pub title: String,
    pub domain: String,
    pub output_index: u32,
    #[serde(default)]
    pub start_index: Option<u32>,
    #[serde(default)]
    pub end_index: Option<u32>,
    pub retrieved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSourceDraft {
    pub research_run_id: Uuid,
    pub response_id: String,
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
    pub domain: String,
    pub retrieved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenAiUsageTotals {
    pub today_cost_usd: f64,
    pub month_cost_usd: f64,
    pub today_request_count: u64,
    pub month_request_count: u64,
}
