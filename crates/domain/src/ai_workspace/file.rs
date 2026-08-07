use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
