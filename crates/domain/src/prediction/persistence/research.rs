use super::super::P4Horizon;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchRunStatus {
    Planned,
    Running,
    Succeeded,
    Partial,
    Failed,
    Cancelled,
}

impl ResearchRunStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRunDraft {
    pub match_id: Uuid,
    pub horizon: P4Horizon,
    pub data_cutoff_at: DateTime<Utc>,
    pub trace_id: Uuid,
    pub idempotency_key: String,
    #[serde(default)]
    pub planner_version: Option<String>,
    #[serde(default)]
    pub prompt_version_id: Option<Uuid>,
    pub schema_version_id: Uuid,
    #[serde(default)]
    pub request_payload: Value,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRunRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub horizon: P4Horizon,
    pub data_cutoff_at: DateTime<Utc>,
    pub trace_id: Uuid,
    pub idempotency_key: String,
    pub request_fingerprint: String,
    pub status: ResearchRunStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRunEventDraft {
    pub research_run_id: Uuid,
    pub idempotency_key: String,
    pub status: ResearchRunStatus,
    #[serde(default)]
    pub response_id: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub token_usage: Value,
    #[serde(default)]
    pub error_category: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub payload: Value,
}
