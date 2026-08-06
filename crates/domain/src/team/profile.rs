use super::{default_tactical_style, default_team_profile_confidence, default_team_type};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamProfileDraft {
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default = "default_team_type")]
    pub team_type: String,
    #[serde(default)]
    pub founded_year: Option<i16>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub stadium: Option<String>,
    #[serde(default)]
    pub head_coach: Option<String>,
    #[serde(default)]
    pub default_formation: Option<String>,
    #[serde(default = "default_tactical_style")]
    pub tactical_style: String,
    #[serde(default)]
    pub attack_rating: Option<f64>,
    #[serde(default)]
    pub midfield_rating: Option<f64>,
    #[serde(default)]
    pub defence_rating: Option<f64>,
    #[serde(default)]
    pub goalkeeper_rating: Option<f64>,
    #[serde(default)]
    pub reputation: Option<f64>,
    #[serde(default = "default_team_profile_confidence")]
    pub data_confidence: f64,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamProfileRecord {
    pub team_id: Uuid,
    pub short_name: Option<String>,
    pub team_type: String,
    pub founded_year: Option<i16>,
    pub city: Option<String>,
    pub stadium: Option<String>,
    pub head_coach: Option<String>,
    pub default_formation: Option<String>,
    pub tactical_style: String,
    pub attack_rating: Option<f64>,
    pub midfield_rating: Option<f64>,
    pub defence_rating: Option<f64>,
    pub goalkeeper_rating: Option<f64>,
    pub reputation: Option<f64>,
    pub data_confidence: f64,
    pub notes: Option<String>,
    pub metadata: Value,
    pub updated_at: DateTime<Utc>,
}
