use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const PREDICTION_INPUT_AUDIT_VERSION: &str = "prematch-input-audit-v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PredictionReadinessLevel {
    FormalReady,
    ReadyWithWarnings,
    ShadowOnly,
    Blocked,
}

impl PredictionReadinessLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FormalReady => "formal_ready",
            Self::ReadyWithWarnings => "ready_with_warnings",
            Self::ShadowOnly => "shadow_only",
            Self::Blocked => "blocked",
        }
    }

    pub const fn can_run_formal(self) -> bool {
        matches!(self, Self::FormalReady | Self::ReadyWithWarnings)
    }

    pub const fn can_run_shadow(self) -> bool {
        !matches!(self, Self::Blocked)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PredictionReadinessCheckStatus {
    Passed,
    Warning,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionReadinessCheck {
    pub code: String,
    pub label: String,
    pub status: PredictionReadinessCheckStatus,
    pub weight: u8,
    pub score: u8,
    pub summary: String,
    #[serde(default)]
    pub details: Vec<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPredictionReadiness {
    pub audit_version: String,
    pub match_id: Uuid,
    pub match_key: String,
    pub snapshot_type: String,
    pub model_family: String,
    pub assessed_at: DateTime<Utc>,
    pub data_cutoff_at: Option<DateTime<Utc>>,
    pub level: PredictionReadinessLevel,
    pub score: u8,
    pub can_run_formal: bool,
    pub can_run_shadow: bool,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub checks: Vec<PredictionReadinessCheck>,
    #[serde(default)]
    pub input_manifest: Option<Value>,
    #[serde(default)]
    pub input_manifest_sha256: Option<String>,
    #[serde(default)]
    pub route_identity: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionInputAuditSummary {
    pub audit_version: String,
    pub readiness_level: String,
    pub readiness_score: Option<u8>,
    pub input_manifest_sha256: String,
    pub input_sha256: String,
}
