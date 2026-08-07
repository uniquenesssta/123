use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const POSTMATCH_SETTLEMENT_VERSION: &str = "postmatch-settlement-v1";
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
