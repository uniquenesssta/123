use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AbilityCandidateStatus {
    Pending,
    Accepted,
    Rejected,
    Superseded,
}

impl AbilityCandidateStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AbilityCandidateDecision {
    Accept,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResultDraft {
    pub match_id: Uuid,
    pub home_goals_90: i16,
    pub away_goals_90: i16,
    #[serde(default)]
    pub home_goals_extra_time: Option<i16>,
    #[serde(default)]
    pub away_goals_extra_time: Option<i16>,
    #[serde(default)]
    pub home_penalties: Option<i16>,
    #[serde(default)]
    pub away_penalties: Option<i16>,
    pub finalized_at: DateTime<Utc>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResultRecord {
    pub match_id: Uuid,
    pub home_goals_90: i16,
    pub away_goals_90: i16,
    pub home_goals_extra_time: Option<i16>,
    pub away_goals_extra_time: Option<i16>,
    pub home_penalties: Option<i16>,
    pub away_penalties: Option<i16>,
    pub finalized_at: DateTime<Utc>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstitutionDraft {
    pub team_id: Uuid,
    #[serde(default)]
    pub player_out_id: Option<Uuid>,
    #[serde(default)]
    pub player_in_id: Option<Uuid>,
    pub minute: i16,
    #[serde(default = "default_period")]
    pub period: String,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_period() -> String {
    "normal_time".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstitutionRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub player_out_id: Option<Uuid>,
    pub player_out_name: Option<String>,
    pub player_in_id: Option<Uuid>,
    pub player_in_name: Option<String>,
    pub minute: i16,
    pub period: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerPerformanceMetrics {
    #[serde(default)]
    pub goals: f64,
    #[serde(default)]
    pub assists: f64,
    #[serde(default)]
    pub expected_goals: f64,
    #[serde(default)]
    pub expected_assists: f64,
    #[serde(default)]
    pub shots: f64,
    #[serde(default)]
    pub shots_on_target: f64,
    #[serde(default)]
    pub key_passes: f64,
    #[serde(default)]
    pub progressive_actions: f64,
    #[serde(default)]
    pub tackles: f64,
    #[serde(default)]
    pub interceptions: f64,
    #[serde(default)]
    pub clearances: f64,
    #[serde(default)]
    pub blocks: f64,
    #[serde(default)]
    pub duels_won: f64,
    #[serde(default)]
    pub duels_total: f64,
    #[serde(default)]
    pub fouls: f64,
    #[serde(default)]
    pub yellow_cards: f64,
    #[serde(default)]
    pub red_cards: f64,
    #[serde(default)]
    pub errors_leading_to_shot: f64,
    #[serde(default)]
    pub provider_rating: Option<f64>,
    #[serde(default)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMatchObservationDraft {
    pub player_id: Uuid,
    pub team_id: Uuid,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub role_code: Option<String>,
    pub started: bool,
    pub minutes_played: i16,
    #[serde(default)]
    pub performance_score: Option<f64>,
    #[serde(default = "default_confidence")]
    pub input_confidence: f64,
    #[serde(default)]
    pub metrics: PlayerPerformanceMetrics,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
}

fn default_confidence() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMatchObservationRecord {
    pub id: Uuid,
    pub match_id: Uuid,
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
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewDraft {
    pub match_id: Uuid,
    #[serde(default)]
    pub review_version: Option<String>,
    #[serde(default = "default_data_coverage")]
    pub data_coverage: f64,
    #[serde(default)]
    pub source_run_id: Option<Uuid>,
    pub result: MatchResultDraft,
    #[serde(default)]
    pub substitutions: Vec<SubstitutionDraft>,
    #[serde(default)]
    pub events: Vec<crate::MatchReviewEventDraft>,
    pub player_observations: Vec<PlayerMatchObservationDraft>,
    #[serde(default)]
    pub notes: Option<String>,
}

fn default_data_coverage() -> f64 {
    1.0
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewSummary {
    pub id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub review_version: String,
    pub status: String,
    pub data_coverage: f64,
    pub source_run_id: Option<Uuid>,
    pub calculation_version: String,
    pub result_snapshot: Value,
    pub substitutions_snapshot: Value,
    pub prediction_evaluation: Value,
    pub conclusions: Value,
    pub created_at: DateTime<Utc>,
    pub finalized_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityUpdateCandidateRecord {
    pub id: Uuid,
    pub match_review_id: Option<Uuid>,
    pub player_match_review_id: Option<Uuid>,
    pub player_id: Uuid,
    pub player_name: String,
    pub dimension_code: String,
    pub dimension_name: String,
    pub current_value: Option<f64>,
    pub proposed_value: f64,
    pub confidence: f64,
    pub sample_size: i32,
    pub evidence: Value,
    pub calculation_version: String,
    pub status: AbilityCandidateStatus,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decided_by: Option<String>,
    pub decision_note: Option<String>,
    pub accepted_observation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityCandidateDecisionDraft {
    pub candidate_id: Uuid,
    pub decision: AbilityCandidateDecision,
    #[serde(default)]
    pub decided_by: Option<String>,
    #[serde(default)]
    pub decision_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewEventRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub event_key: String,
    pub sequence_no: i32,
    pub event_type: crate::MatchEventType,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub player_id: Option<Uuid>,
    pub player_name: Option<String>,
    pub related_player_id: Option<Uuid>,
    pub related_player_name: Option<String>,
    pub minute: i16,
    pub stoppage_minute: Option<i16>,
    pub period: String,
    pub home_score: Option<i16>,
    pub away_score: Option<i16>,
    pub verification_status: crate::MatchEventVerificationStatus,
    pub revision_status: crate::MatchEventRevisionStatus,
    pub verified_at: Option<DateTime<Utc>>,
    pub source_document_id: Option<Uuid>,
    pub source_package_id: Option<Uuid>,
    pub revision_of_event_id: Option<Uuid>,
    pub description: Option<String>,
    pub source_urls: Vec<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub recorded_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewDetail {
    pub summary: MatchReviewSummary,
    pub result: MatchResultRecord,
    pub substitutions: Vec<SubstitutionRecord>,
    pub events: Vec<MatchReviewEventRecord>,
    pub event_summary: crate::MatchEventSummary,
    pub player_reviews: Vec<PlayerMatchReviewRecord>,
    pub team_reviews: Vec<TeamMatchReviewRecord>,
    pub ability_candidates: Vec<AbilityUpdateCandidateRecord>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityCandidateProposal {
    pub player_id: Uuid,
    pub dimension_code: String,
    pub current_value: Option<f64>,
    pub proposed_value: f64,
    pub confidence: f64,
    pub sample_size: i32,
    pub evidence: Value,
}

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
