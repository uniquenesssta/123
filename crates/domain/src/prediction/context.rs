use crate::CompetitionKind;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchContext {
    pub match_key: String,
    pub kickoff_time: DateTime<Utc>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    #[serde(default)]
    pub competition_kind: CompetitionKind,
    pub home_team_name: String,
    pub away_team_name: String,
    #[serde(default)]
    pub metadata: Value,
}
