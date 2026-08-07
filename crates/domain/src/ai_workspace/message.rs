use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
