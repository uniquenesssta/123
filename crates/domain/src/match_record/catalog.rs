use super::MatchStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchDraft {
    #[serde(default)]
    pub external_key: String,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    #[serde(default)]
    pub round_id: Option<Uuid>,
    pub home_team_id: Uuid,
    pub away_team_id: Uuid,
    pub kickoff_time: DateTime<Utc>,
    #[serde(default)]
    pub status: MatchStatus,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRecord {
    pub id: Uuid,
    pub external_key: String,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub round_id: Option<Uuid>,
    pub home_team_id: Uuid,
    pub home_team_name: String,
    pub away_team_id: Uuid,
    pub away_team_name: String,
    pub kickoff_time: DateTime<Utc>,
    pub status: MatchStatus,
    pub venue: Option<String>,
}
