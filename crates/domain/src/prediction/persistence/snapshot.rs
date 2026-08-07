use super::super::P4Horizon;
use super::EvidenceVerificationState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
