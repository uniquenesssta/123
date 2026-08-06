use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonDraft {
    pub competition_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub starts_on: Option<NaiveDate>,
    #[serde(default)]
    pub ends_on: Option<NaiveDate>,
    #[serde(default = "default_season_status")]
    pub status: String,
    #[serde(default)]
    pub metadata: Value,
}

fn default_season_status() -> String {
    "planned".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonRecord {
    pub id: Uuid,
    pub competition_id: Uuid,
    pub competition_name: String,
    pub name: String,
    pub starts_on: Option<NaiveDate>,
    pub ends_on: Option<NaiveDate>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonTeamMembershipOption {
    pub season_id: Uuid,
    pub team_id: Uuid,
    pub registration_status: String,
}
