use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
