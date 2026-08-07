use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMatchReviewRecord {
    pub id: Uuid,
    pub match_review_id: Uuid,
    pub player_id: Uuid,
    pub player_name: String,
    pub team_id: Uuid,
    pub team_name: String,
    pub role_code: Option<String>,
    pub started: bool,
    pub entry_type: String,
    pub minutes_played: Option<i16>,
    pub expected_performance: Option<f64>,
    pub actual_performance: Option<f64>,
    pub realization_ratio: Option<f64>,
    pub confidence: f64,
    pub contribution_weight: f64,
    pub ability_candidate_count: i32,
    pub metrics: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMatchReviewRecord {
    pub id: Uuid,
    pub match_review_id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub chemistry_score: Option<f64>,
    pub lineup_continuity: Option<f64>,
    pub performance_cohesion: Option<f64>,
    pub bench_strength: Option<f64>,
    pub bench_dropoff: Option<f64>,
    pub substitution_impact: Option<f64>,
    pub substitute_count: i32,
    pub realization_score: Option<f64>,
    pub confidence: f64,
    pub metrics: Value,
}
