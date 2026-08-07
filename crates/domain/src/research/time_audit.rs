use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TimeAuditStatus {
    Accepted,
    AcceptedNonFact,
    RejectedFuture,
    RejectedRetrievedAfterCutoff,
    RejectedMissingEvidenceTime,
    RejectedMissingTimezone,
    RejectedInvalidOrder,
}

impl TimeAuditStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::AcceptedNonFact => "accepted_non_fact",
            Self::RejectedFuture => "rejected_future",
            Self::RejectedRetrievedAfterCutoff => "rejected_retrieved_after_cutoff",
            Self::RejectedMissingEvidenceTime => "rejected_missing_evidence_time",
            Self::RejectedMissingTimezone => "rejected_missing_timezone",
            Self::RejectedInvalidOrder => "rejected_invalid_order",
        }
    }

    pub const fn accepted(self) -> bool {
        matches!(self, Self::Accepted | Self::AcceptedNonFact)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAuditDraft {
    pub research_run_id: Uuid,
    pub match_id: Uuid,
    pub trace_id: Uuid,
    pub fact_key: String,
    pub field_key: String,
    pub data_cutoff_at: DateTime<Utc>,
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub observed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub effective_at: Option<DateTime<Utc>>,
    pub retrieved_at: DateTime<Utc>,
    #[serde(default)]
    pub timezone: Option<String>,
    pub status: TimeAuditStatus,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAuditRecord {
    pub id: Uuid,
    pub status: TimeAuditStatus,
    pub time_fingerprint: String,
    pub created_at: DateTime<Utc>,
}
