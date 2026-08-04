use crate::{ApiProtocol, GatewayError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAiConnectionTest {
    pub model_id: String,
    pub protocol: ApiProtocol,
    pub endpoint_url: String,
    pub provider_request_id: Option<String>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayOperation {
    Research,
    Extraction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GatewayRequest {
    pub operation: GatewayOperation,
    pub trace_id: String,
    pub match_key: String,
    pub data_cutoff_at: DateTime<Utc>,
    pub schema_name: String,
    pub schema_version: String,
    pub schema: Value,
    pub static_instructions: String,
    pub dynamic_context: Value,
    pub requested_fact_keys: Vec<String>,
    #[serde(default)]
    pub daily_spend_usd: f64,
    #[serde(default)]
    pub monthly_spend_usd: f64,
    #[serde(default)]
    pub attempt_number_offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StructuredGatewayRequest {
    pub operation: GatewayOperation,
    pub trace_id: String,
    pub schema_name: String,
    pub schema_version: String,
    pub schema: Value,
    pub static_instructions: String,
    pub input: Value,
    #[serde(default)]
    pub enable_web_search: bool,
    #[serde(default)]
    pub daily_spend_usd: f64,
    #[serde(default)]
    pub monthly_spend_usd: f64,
    #[serde(default)]
    pub attempt_number_offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredGatewayResponse {
    pub response_id: String,
    pub model_id: String,
    pub status: String,
    pub output: Value,
    pub citations: Vec<WebCitation>,
    pub sources: Vec<WebSource>,
    pub usage: GatewayUsage,
    pub search_call_count: u32,
    pub provider_request_id: Option<String>,
    pub raw_response: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StructuredGatewayExecution {
    pub response: StructuredGatewayResponse,
    pub attempts: Vec<GatewayAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PlainTextMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlainTextGatewayRequest {
    pub operation: GatewayOperation,
    pub trace_id: String,
    pub static_instructions: String,
    pub messages: Vec<PlainTextMessage>,
    #[serde(default)]
    pub daily_spend_usd: f64,
    #[serde(default)]
    pub monthly_spend_usd: f64,
    #[serde(default)]
    pub attempt_number_offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlainTextGatewayResponse {
    pub response_id: String,
    pub model_id: String,
    pub status: String,
    pub text: String,
    pub usage: GatewayUsage,
    pub provider_request_id: Option<String>,
    pub raw_response: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlainTextGatewayExecution {
    pub response: PlainTextGatewayResponse,
    pub attempts: Vec<GatewayAttempt>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchValueKind {
    String,
    Number,
    Integer,
    Boolean,
    StringList,
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResearchValue {
    pub kind: ResearchValueKind,
    pub text: Option<String>,
    pub number: Option<f64>,
    pub integer: Option<i64>,
    pub boolean: Option<bool>,
    pub strings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResearchSubject {
    pub entity_type: String,
    pub name: String,
    pub external_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResearchFact {
    pub fact_key: String,
    pub field_key: String,
    pub subject: ResearchSubject,
    pub value: ResearchValue,
    pub verification_state: String,
    pub source_urls: Vec<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub observed_at: Option<DateTime<Utc>>,
    pub effective_at: Option<DateTime<Utc>>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MissingField {
    pub field_key: String,
    pub verification_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ResearchOutput {
    pub schema_version: String,
    pub match_key: String,
    pub data_cutoff_at: DateTime<Utc>,
    pub facts: Vec<ResearchFact>,
    pub missing_fields: Vec<MissingField>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CitationLocation {
    pub output_index: usize,
    pub start_index: Option<usize>,
    pub end_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebCitation {
    pub url: String,
    pub title: String,
    pub domain: String,
    pub location: CitationLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WebSource {
    pub url: String,
    pub title: Option<String>,
    pub domain: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct GatewayUsage {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub output_tokens: u64,
    pub total_tokens: u64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GatewayResponse {
    pub response_id: String,
    pub model_id: String,
    pub status: String,
    pub output: ResearchOutput,
    pub citations: Vec<WebCitation>,
    pub sources: Vec<WebSource>,
    pub usage: GatewayUsage,
    pub search_call_count: u32,
    pub provider_request_id: Option<String>,
    pub raw_response: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GatewayExecution {
    pub response: GatewayResponse,
    pub attempts: Vec<GatewayAttempt>,
}
