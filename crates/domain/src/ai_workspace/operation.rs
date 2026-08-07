use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceOperationDraft {
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub proposal_key: String,
    pub operation_type: String,
    pub payload: Value,
    pub rationale: String,
    pub confidence: f64,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceOperationRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub proposal_key: String,
    pub operation_type: String,
    pub payload: Value,
    pub rationale: String,
    pub confidence: f64,
    pub status: String,
    pub result: Value,
    pub error_message: Option<String>,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceApplyResult {
    pub operation_id: Uuid,
    pub operation_type: String,
    pub status: String,
    pub result: Value,
    pub error_message: Option<String>,
}
