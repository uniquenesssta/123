use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AbilityCandidateStatus {
    Pending,
    Accepted,
    Rejected,
    Superseded,
}
impl AbilityCandidateStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AbilityCandidateDecision {
    Accept,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityUpdateCandidateRecord {
    pub id: Uuid,
    pub match_review_id: Option<Uuid>,
    pub player_match_review_id: Option<Uuid>,
    pub player_id: Uuid,
    pub player_name: String,
    pub dimension_code: String,
    pub dimension_name: String,
    pub current_value: Option<f64>,
    pub proposed_value: f64,
    pub confidence: f64,
    pub sample_size: i32,
    pub evidence: Value,
    pub calculation_version: String,
    pub status: AbilityCandidateStatus,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decided_by: Option<String>,
    pub decision_note: Option<String>,
    pub accepted_observation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityCandidateDecisionDraft {
    pub candidate_id: Uuid,
    pub decision: AbilityCandidateDecision,
    #[serde(default)]
    pub decided_by: Option<String>,
    #[serde(default)]
    pub decision_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityCandidateProposal {
    pub player_id: Uuid,
    pub dimension_code: String,
    pub current_value: Option<f64>,
    pub proposed_value: f64,
    pub confidence: f64,
    pub sample_size: i32,
    pub evidence: Value,
}
