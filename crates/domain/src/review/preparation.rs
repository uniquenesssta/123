use super::{MatchResultRecord, MatchReviewSummary, PlayerPerformanceMetrics, SubstitutionRecord};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewableMatch {
    pub match_record: crate::MatchRecord,
    pub result: Option<MatchResultRecord>,
    pub latest_review: Option<MatchReviewSummary>,
    pub player_observation_count: i64,
    pub actual_lineup_count: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPlayerBaseline {
    pub observation_id: Uuid,
    pub player_id: Uuid,
    pub player_name: String,
    pub team_id: Uuid,
    pub team_name: String,
    pub position_code: Option<String>,
    pub role_code: Option<String>,
    pub started: bool,
    pub minutes_played: i16,
    pub performance_score: Option<f64>,
    pub input_confidence: f64,
    pub metrics: PlayerPerformanceMetrics,
    pub expected_performance: f64,
    pub expected_confidence: f64,
    pub current_abilities: Value,
    pub reviewed_match_count: i32,
    pub substitute_appearances: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewTeamContext {
    pub team_id: Uuid,
    pub team_name: String,
    pub recent_starter_overlap: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionReviewContext {
    pub run_id: Uuid,
    pub summary: Value,
    pub actual_scoreline_probability: Option<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewPreparationData {
    pub data_coverage: f64,
    pub match_record: crate::MatchRecord,
    pub result: MatchResultRecord,
    pub substitutions: Vec<SubstitutionRecord>,
    pub players: Vec<ReviewPlayerBaseline>,
    pub teams: Vec<ReviewTeamContext>,
    pub prediction: Option<PredictionReviewContext>,
}
