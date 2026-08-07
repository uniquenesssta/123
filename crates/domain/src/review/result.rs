use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResultDraft {
    pub match_id: Uuid, pub home_goals_90: i16, pub away_goals_90: i16,
    #[serde(default)] pub home_goals_extra_time: Option<i16>,
    #[serde(default)] pub away_goals_extra_time: Option<i16>,
    #[serde(default)] pub home_penalties: Option<i16>,
    #[serde(default)] pub away_penalties: Option<i16>,
    pub finalized_at: DateTime<Utc>,
    #[serde(default)] pub source_document_id: Option<Uuid>,
    #[serde(default)] pub metadata: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResultRecord {
    pub match_id: Uuid, pub home_goals_90: i16, pub away_goals_90: i16,
    pub home_goals_extra_time: Option<i16>, pub away_goals_extra_time: Option<i16>,
    pub home_penalties: Option<i16>, pub away_penalties: Option<i16>,
    pub finalized_at: DateTime<Utc>, pub metadata: Value,
}
