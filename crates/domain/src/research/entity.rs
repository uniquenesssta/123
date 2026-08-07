use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EntityResolutionStatus {
    Resolved,
    Ambiguous,
    Unmatched,
    Unsupported,
}

impl EntityResolutionStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Ambiguous => "ambiguous",
            Self::Unmatched => "unmatched",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityCandidate {
    pub entity_id: Uuid,
    pub canonical_name: String,
    pub matched_name: String,
    pub strategy: String,
    pub score: u16,
    #[serde(default)]
    pub relation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResolutionDraft {
    pub research_run_id: Uuid,
    pub match_id: Uuid,
    pub trace_id: Uuid,
    pub fact_key: String,
    pub entity_type: String,
    pub raw_name: String,
    pub normalized_name: String,
    #[serde(default)]
    pub external_id: Option<String>,
    pub status: EntityResolutionStatus,
    #[serde(default)]
    pub resolved_entity_id: Option<Uuid>,
    #[serde(default)]
    pub resolved_name: Option<String>,
    pub strategy: String,
    pub confidence_score: u16,
    #[serde(default)]
    pub candidates: Vec<EntityCandidate>,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResolutionRecord {
    pub id: Uuid,
    pub status: EntityResolutionStatus,
    pub resolved_entity_id: Option<Uuid>,
    pub resolution_fingerprint: String,
    pub created_at: DateTime<Utc>,
}
