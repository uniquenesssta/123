use super::AbilityCandidateProposal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedPlayerReview {
    pub observation_id: Uuid,
    pub player_id: Uuid,
    pub team_id: Uuid,
    pub role_code: Option<String>,
    pub started: bool,
    pub entry_type: String,
    pub minutes_played: i16,
    pub expected_performance: f64,
    pub actual_performance: f64,
    pub realization_ratio: f64,
    pub confidence: f64,
    pub contribution_weight: f64,
    pub metrics: Value,
    pub ability_candidates: Vec<AbilityCandidateProposal>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedTeamReview {
    pub team_id: Uuid,
    pub chemistry_score: f64,
    pub lineup_continuity: f64,
    pub performance_cohesion: f64,
    pub bench_strength: f64,
    pub bench_dropoff: f64,
    pub substitution_impact: f64,
    pub substitute_count: i32,
    pub realization_score: f64,
    pub confidence: f64,
    pub metrics: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedMatchReview {
    pub calculation_version: String,
    pub prediction_evaluation: Value,
    pub conclusions: Value,
    pub player_reviews: Vec<CalculatedPlayerReview>,
    pub team_reviews: Vec<CalculatedTeamReview>,
}
