use crate::{GatewayError, GatewayUsage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GatewayAttempt {
    pub attempt_number: u32,
    pub model_id: String,
    pub request_fingerprint: String,
    pub request_payload: Value,
    pub response_id: Option<String>,
    pub provider_request_id: Option<String>,
    pub provider_status: Option<u16>,
    pub status: String,
    pub usage: GatewayUsage,
    pub latency_ms: u64,
    pub search_call_count: u32,
    pub estimated_cost_usd: Option<f64>,
    pub raw_response: Option<Value>,
    pub error: Option<GatewayError>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}
