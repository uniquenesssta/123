use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use super::{AbilityUpdateCandidateRecord, MatchEventSummary, MatchResultDraft, MatchResultRecord, MatchReviewEventDraft, MatchReviewEventRecord, PlayerMatchObservationDraft, PlayerMatchReviewRecord, SubstitutionDraft, SubstitutionRecord, TeamMatchReviewRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewDraft {
    pub match_id: Uuid,
    #[serde(default)] pub review_version: Option<String>,
    #[serde(default = "default_data_coverage")] pub data_coverage: f64,
    #[serde(default)] pub source_run_id: Option<Uuid>,
    pub result: MatchResultDraft,
    #[serde(default)] pub substitutions: Vec<SubstitutionDraft>,
    #[serde(default)] pub events: Vec<MatchReviewEventDraft>,
    pub player_observations: Vec<PlayerMatchObservationDraft>,
    #[serde(default)] pub notes: Option<String>,
}
fn default_data_coverage() -> f64 { 1.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewSummary {
    pub id: Uuid, pub match_id: Uuid, pub match_key: String, pub home_team_name: String, pub away_team_name: String,
    pub review_version: String, pub status: String, pub data_coverage: f64, pub source_run_id: Option<Uuid>,
    pub calculation_version: String, pub result_snapshot: Value, pub substitutions_snapshot: Value,
    pub prediction_evaluation: Value, pub conclusions: Value, pub created_at: DateTime<Utc>,
    pub finalized_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewDetail {
    pub summary: MatchReviewSummary, pub result: MatchResultRecord, pub substitutions: Vec<SubstitutionRecord>,
    pub events: Vec<MatchReviewEventRecord>, pub event_summary: MatchEventSummary,
    pub player_reviews: Vec<PlayerMatchReviewRecord>, pub team_reviews: Vec<TeamMatchReviewRecord>,
    pub ability_candidates: Vec<AbilityUpdateCandidateRecord>,
}
