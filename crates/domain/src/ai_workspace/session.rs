use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::file::ApiWorkspaceGeneratedFileRecord;
use super::message::ApiWorkspaceMessageRecord;
use super::operation::ApiWorkspaceOperationRecord;

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
pub struct ApiWorkspaceSessionDetail {
    pub session: ApiWorkspaceSessionRecord,
    pub messages: Vec<ApiWorkspaceMessageRecord>,
    pub operations: Vec<ApiWorkspaceOperationRecord>,
    pub files: Vec<ApiWorkspaceGeneratedFileRecord>,
}
