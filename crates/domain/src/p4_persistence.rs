use crate::CompetitionKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const P4_PERSISTENCE_CONTRACT_VERSION: &str = "football.p4-persistence.v1";
pub const P4_EVIDENCE_SCHEMA_VERSION: &str = "football.p4-evidence.v1";
pub const P4_SNAPSHOT_SCHEMA_VERSION: &str = "football.p4-prematch-snapshot.v1";
pub const P4_FEATURE_FIELD_COUNT: usize = 31;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum P4Horizon {
    #[serde(rename = "T-24h")]
    T24h,
    #[serde(rename = "T-6h")]
    T6h,
    #[serde(rename = "T-90m")]
    T90m,
    #[serde(rename = "T-1h")]
    T1h,
    #[serde(rename = "T-N")]
    LegacyTN,
}

impl P4Horizon {
    pub const CANONICAL: [Self; 3] = [Self::T24h, Self::T6h, Self::T1h];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::T24h => "T-24h",
            Self::T6h => "T-6h",
            Self::T90m => "T-90m",
            Self::T1h => "T-1h",
            Self::LegacyTN => "T-N",
        }
    }

    pub const fn is_canonical(self) -> bool {
        matches!(self, Self::T24h | Self::T6h | Self::T1h)
    }
}

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResearchRunStatus {
    Planned,
    Running,
    Succeeded,
    Partial,
    Failed,
    Cancelled,
}

impl ResearchRunStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSourceKind {
    Real,
    Manual,
    SyntheticFixture,
}

impl SnapshotSourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Real => "real",
            Self::Manual => "manual",
            Self::SyntheticFixture => "synthetic_fixture",
        }
    }

    pub const fn evidence_scope(self) -> &'static str {
        match self {
            Self::Real | Self::Manual => "real",
            Self::SyntheticFixture => "synthetic",
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRunDraft {
    pub match_id: Uuid,
    pub horizon: P4Horizon,
    pub data_cutoff_at: DateTime<Utc>,
    pub trace_id: Uuid,
    pub idempotency_key: String,
    #[serde(default)]
    pub planner_version: Option<String>,
    #[serde(default)]
    pub prompt_version_id: Option<Uuid>,
    pub schema_version_id: Uuid,
    #[serde(default)]
    pub request_payload: Value,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRunRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub horizon: P4Horizon,
    pub data_cutoff_at: DateTime<Utc>,
    pub trace_id: Uuid,
    pub idempotency_key: String,
    pub request_fingerprint: String,
    pub status: ResearchRunStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchRunEventDraft {
    pub research_run_id: Uuid,
    pub idempotency_key: String,
    pub status: ResearchRunStatus,
    #[serde(default)]
    pub response_id: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub token_usage: Value,
    #[serde(default)]
    pub error_category: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub payload: Value,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotFeatureDraft {
    pub field_order: u8,
    pub field_key: String,
    pub value: Value,
    pub verification_state: EvidenceVerificationState,
    #[serde(default)]
    pub evidence_ids: Vec<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotProbabilityDraft {
    pub chain_key: String,
    pub home_win: f64,
    pub draw: f64,
    pub away_win: f64,
    #[serde(default)]
    pub btts: Option<f64>,
    #[serde(default)]
    pub over_2_5: Option<f64>,
    #[serde(default)]
    pub clean_sheet_home: Option<f64>,
    #[serde(default)]
    pub clean_sheet_away: Option<f64>,
    pub matrix_sha256: String,
    #[serde(default = "default_matrix_cell_count")]
    pub matrix_cell_count: u16,
    #[serde(default)]
    pub metadata: Value,
}

fn default_matrix_cell_count() -> u16 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrematchSnapshotDraft {
    pub match_id: Uuid,
    pub match_key: String,
    pub horizon: P4Horizon,
    pub data_cutoff_at: DateTime<Utc>,
    pub frozen_at: DateTime<Utc>,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub competition_profile_id: Uuid,
    #[serde(default)]
    pub research_run_id: Option<Uuid>,
    pub schema_version_id: Uuid,
    pub schema_version: String,
    pub trace_id: Uuid,
    pub idempotency_key: String,
    pub source_kind: SnapshotSourceKind,
    pub quality_score: f64,
    pub input_payload: Value,
    pub features: Vec<SnapshotFeatureDraft>,
    pub probabilities: Vec<SnapshotProbabilityDraft>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrematchSnapshotRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub horizon: P4Horizon,
    pub data_cutoff_at: DateTime<Utc>,
    pub frozen_at: DateTime<Utc>,
    pub snapshot_fingerprint: String,
    pub idempotency_key: String,
    pub source_kind: SnapshotSourceKind,
    pub evidence_scope: String,
    pub created: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrematchSnapshotBundle {
    pub snapshot: PrematchSnapshotRecord,
    pub input_payload: Value,
    pub features: Vec<SnapshotFeatureDraft>,
    pub probabilities: Vec<SnapshotProbabilityDraft>,
}
