use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const API_WORKSPACE_SCHEMA_VERSION: &str = "football.api-workspace-response.v2";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiWorkspacePreset {
    pub key: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub web_search_enabled: bool,
    pub requires_match: bool,
    pub allowed_operation_types: Vec<String>,
    pub suggested_questions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceAttachment {
    pub name: String,
    pub media_type: String,
    pub content: String,
    pub content_sha256: String,
    pub original_size_bytes: u64,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceSessionDraft {
    pub profile_id: String,
    pub preset_key: String,
    pub title: String,
    #[serde(default)]
    pub match_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceSessionRecord {
    pub id: Uuid,
    pub profile_id: String,
    pub preset_key: String,
    pub title: String,
    pub match_id: Option<Uuid>,
    pub match_label: Option<String>,
    pub metadata: Value,
    pub status: String,
    pub message_count: i64,
    pub pending_operation_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceMessageDraft {
    pub session_id: Uuid,
    pub role: String,
    pub content: String,
    #[serde(default)]
    pub structured_payload: Value,
    #[serde(default)]
    pub citations: Value,
    #[serde(default)]
    pub attachments: Value,
    #[serde(default)]
    pub provider_response_id: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub token_usage: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceMessageRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: String,
    pub content: String,
    pub structured_payload: Value,
    pub citations: Value,
    pub attachments: Value,
    pub provider_response_id: Option<String>,
    pub model_id: Option<String>,
    pub token_usage: Value,
    pub created_at: DateTime<Utc>,
}

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
pub struct ApiWorkspaceGeneratedFileDraft {
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub filename: String,
    pub media_type: String,
    pub content: String,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceGeneratedFileRecord {
    pub id: Uuid,
    pub session_id: Uuid,
    pub message_id: Uuid,
    pub filename: String,
    pub media_type: String,
    pub content_sha256: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceGeneratedFileContent {
    pub file: ApiWorkspaceGeneratedFileRecord,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceSessionDetail {
    pub session: ApiWorkspaceSessionRecord,
    pub messages: Vec<ApiWorkspaceMessageRecord>,
    pub operations: Vec<ApiWorkspaceOperationRecord>,
    pub files: Vec<ApiWorkspaceGeneratedFileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceAssistantOperation {
    pub proposal_key: String,
    pub operation_type: String,
    pub payload: Value,
    pub rationale: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceAssistantFile {
    pub filename: String,
    pub media_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiWorkspaceAssistantOutput {
    pub schema_version: String,
    pub answer: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub missing_information: Vec<String>,
    pub warnings: Vec<String>,
    pub proposed_operations: Vec<ApiWorkspaceAssistantOperation>,
    pub generated_files: Vec<ApiWorkspaceAssistantFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceApplyResult {
    pub operation_id: Uuid,
    pub operation_type: String,
    pub status: String,
    pub result: Value,
    pub error_message: Option<String>,
}
