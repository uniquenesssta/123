use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const POSTMATCH_SETTLEMENT_VERSION: &str = "postmatch-settlement-v1";
pub const POSTMATCH_MONITORING_VERSION: &str = "postmatch-monitoring-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchSettlementReadiness {
    pub match_review_id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub result_ready: bool,
    pub finalized_review_ready: bool,
    pub successful_run_ready: bool,
    pub frozen_snapshot_ready: bool,
    pub snapshot_identity_ready: bool,
    pub real_evidence_snapshot_ready: bool,
    pub competition_profile_ready: bool,
    pub formal_horizon_ready: bool,
    pub existing_settlement_id: Option<Uuid>,
    pub blocked_reasons: Vec<String>,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchSettlementDraft {
    pub match_review_id: Uuid,
    #[serde(default)]
    pub settled_by: Option<String>,
    #[serde(default)]
    pub settlement_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchSettlementRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub match_review_id: Uuid,
    pub model_run_id: Uuid,
    pub feature_snapshot_id: Uuid,
    pub competition_id: Uuid,
    pub competition_name: String,
    pub competition_profile_id: Uuid,
    pub model_version_id: Uuid,
    pub model_version: String,
    pub parameter_set_id: Uuid,
    pub parameter_version: String,
    pub rule_package_id: Uuid,
    pub horizon: String,
    pub match_key: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub home_goals_90: i16,
    pub away_goals_90: i16,
    pub result_finalized_at: DateTime<Utc>,
    pub result_fingerprint: String,
    pub settlement_key: String,
    pub settlement_version: String,
    pub status: String,
    pub evidence_item_count: u64,
    pub scored_evidence_count: u64,
    pub drift_status: Option<String>,
    pub settled_by: Option<String>,
    pub settlement_note: Option<String>,
    pub metadata: Value,
    pub settled_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderScoreSnapshotRecord {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub scope_key: String,
    pub competition_id: Uuid,
    pub competition_profile_id: Uuid,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub horizon: String,
    pub sample_size: u64,
    pub correct_count: u64,
    pub partial_count: u64,
    pub incorrect_count: u64,
    pub not_verifiable_count: u64,
    pub accuracy_mean: f64,
    pub timeliness_mean: f64,
    pub reliability_mean: f64,
    pub weighted_score: f64,
    pub decision_set_sha256: String,
    pub calculation_version: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchDriftFindingRecord {
    pub metric_name: String,
    pub baseline_mean: f64,
    pub current_mean: f64,
    pub absolute_delta: f64,
    pub relative_delta: Option<f64>,
    pub severity: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchDriftRunRecord {
    pub id: Uuid,
    pub competition_id: Uuid,
    pub competition_name: String,
    pub competition_profile_id: Uuid,
    pub model_version_id: Uuid,
    pub model_version: String,
    pub parameter_set_id: Uuid,
    pub parameter_version: String,
    pub horizon: String,
    pub partition_key: String,
    pub baseline_size: u64,
    pub current_size: u64,
    pub baseline_window: Value,
    pub current_window: Value,
    pub status: String,
    pub run_key: String,
    pub calculation_version: String,
    pub findings: Vec<PostmatchDriftFindingRecord>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchMonitoringRequest {
    pub competition_id: Uuid,
    pub horizon: String,
    #[serde(default = "default_postmatch_baseline_size")]
    pub baseline_size: usize,
    #[serde(default = "default_postmatch_current_size")]
    pub current_size: usize,
}

fn default_postmatch_baseline_size() -> usize {
    100
}

fn default_postmatch_current_size() -> usize {
    50
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchOverview {
    pub settlement_count: u64,
    pub pending_evidence_count: u64,
    pub scored_evidence_count: u64,
    pub settlements: Vec<PostmatchSettlementRecord>,
    pub evidence_queue: Vec<EvidenceScoringItemRecord>,
    pub provider_scores: Vec<ProviderScoreSnapshotRecord>,
    pub drift_runs: Vec<PostmatchDriftRunRecord>,
}
