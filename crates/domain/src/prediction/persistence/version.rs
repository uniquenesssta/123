use crate::CompetitionKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersionDraft {
    pub schema_key: String,
    pub version: String,
    pub schema_kind: String,
    pub schema_body: Value,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersionRecord {
    pub id: Uuid,
    pub schema_key: String,
    pub version: String,
    pub schema_kind: String,
    pub content_sha256: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersionDraft {
    pub prompt_key: String,
    pub version: String,
    pub prompt_role: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVersionRecord {
    pub id: Uuid,
    pub prompt_key: String,
    pub version: String,
    pub prompt_role: String,
    pub content_sha256: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionProfileVersionDraft {
    pub profile_key: String,
    pub version: String,
    pub name: String,
    pub competition_kind: CompetitionKind,
    pub definition: Value,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionProfileVersionRecord {
    pub id: Uuid,
    pub profile_key: String,
    pub version: String,
    pub definition_sha256: String,
    pub created_at: DateTime<Utc>,
}
