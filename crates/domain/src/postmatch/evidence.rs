use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceVerdict {
    Correct,
    Partial,
    Incorrect,
    NotVerifiable,
}
impl EvidenceVerdict {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Correct => "correct",
            Self::Partial => "partial",
            Self::Incorrect => "incorrect",
            Self::NotVerifiable => "not_verifiable",
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceScoringDecisionDraft {
    pub item_id: Uuid,
    pub verdict: EvidenceVerdict,
    #[serde(default)]
    pub decided_by: Option<String>,
    pub decision_note: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceScoringItemRecord {
    pub id: Uuid,
    pub settlement_id: Uuid,
    pub evidence_id: Uuid,
    pub provider_id: Option<Uuid>,
    pub provider_name: Option<String>,
    pub source_document_id: Option<Uuid>,
    pub field_key: String,
    pub verification_state: String,
    pub source_tier: String,
    pub source_title: Option<String>,
    pub source_domain: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub retrieved_at: DateTime<Utc>,
    pub data_cutoff_at: DateTime<Utc>,
    pub timeliness_score: f64,
    pub decision_id: Option<Uuid>,
    pub verdict: Option<String>,
    pub accuracy_score: Option<f64>,
    pub reliability_score: Option<f64>,
    pub decided_by: Option<String>,
    pub decision_note: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
