use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterReplayFixture {
    pub review_id: Uuid,
    pub run_id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub competition_id: Option<Uuid>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub competition_kind: String,
    pub competition_profile_id: Uuid,
    pub kickoff_time: DateTime<Utc>,
    pub home_team_name: String,
    pub away_team_name: String,
    pub snapshot_type: String,
    pub input_payload: Value,
    pub rule_package_version: Option<String>,
    pub actual_home_goals: i16,
    pub actual_away_goals: i16,
    pub baseline_home_win: f64,
    pub baseline_draw: f64,
    pub baseline_away_win: f64,
    pub baseline_scoreline_probability: Option<f64>,
    pub data_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterShadowValidationRequest {
    pub candidate_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterShadowValidationRecord {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub validation_key: String,
    pub partition_key: String,
    pub sample_count: u64,
    pub baseline_metrics: Value,
    pub candidate_metrics: Value,
    pub metric_deltas: Value,
    pub gate_results: Value,
    pub status: String,
    pub generated_at: DateTime<Utc>,
}
