use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EvidenceVerificationState {
    #[serde(rename = "CONFIRMED")]
    Confirmed,
    #[serde(rename = "PROBABLE")]
    Probable,
    #[serde(rename = "CONFLICT")]
    Conflict,
    #[serde(rename = "NOT_FOUND")]
    NotFound,
    #[serde(rename = "STALE")]
    Stale,
    #[serde(rename = "NOT_APPLICABLE")]
    NotApplicable,
}

impl EvidenceVerificationState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Confirmed => "CONFIRMED",
            Self::Probable => "PROBABLE",
            Self::Conflict => "CONFLICT",
            Self::NotFound => "NOT_FOUND",
            Self::Stale => "STALE",
            Self::NotApplicable => "NOT_APPLICABLE",
        }
    }

    pub const fn requires_source(self) -> bool {
        !matches!(self, Self::NotFound | Self::NotApplicable)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceClaimDraft {
    pub match_id: Uuid,
    pub entity_type: String,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    pub field_key: String,
    pub value: Value,
    pub verification_state: EvidenceVerificationState,
    pub source_tier: String,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub source_title: Option<String>,
    #[serde(default)]
    pub source_domain: Option<String>,
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
    #[serde(default)]
    pub effective_at: Option<DateTime<Utc>>,
    pub retrieved_at: DateTime<Utc>,
    pub timezone: String,
    #[serde(default)]
    pub independent_source_count: u16,
    #[serde(default)]
    pub conflict_group_id: Option<Uuid>,
    pub research_run_id: Uuid,
    #[serde(default)]
    pub prompt_version_id: Option<Uuid>,
    #[serde(default)]
    pub prompt_version: Option<String>,
    pub schema_version_id: Uuid,
    pub schema_version: String,
    pub idempotency_key: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceClaimRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub field_key: String,
    pub verification_state: EvidenceVerificationState,
    pub content_sha256: String,
    pub claim_fingerprint: String,
    pub idempotency_key: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceConflictDraft {
    pub match_id: Uuid,
    pub entity_type: String,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    pub field_key: String,
    pub conflict_key: String,
    pub evidence_ids: Vec<Uuid>,
    pub trace_id: Uuid,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceConflictRecord {
    pub id: Uuid,
    pub conflict_key: String,
    pub created_at: DateTime<Utc>,
}
