use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dynamic_tag::PlayerDynamicTagRecord;

fn default_neutral_tactical_role_confidence() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMatchContributionRequest {
    pub player_id: Uuid,
    #[serde(default)]
    pub match_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub role_code: Option<String>,
    #[serde(default)]
    pub role_origin: Option<String>,
    #[serde(default)]
    pub role_source_position_code: Option<String>,
    #[serde(default)]
    pub opponent_team_id: Option<Uuid>,
    pub as_of: DateTime<Utc>,
    #[serde(default)]
    pub data_cutoff_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub expected_minutes: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionComponent {
    pub code: String,
    pub label: String,
    pub value: f64,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMatchContribution {
    pub player_id: Uuid,
    pub player_name: String,
    pub match_id: Option<Uuid>,
    pub as_of: DateTime<Utc>,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub tactical_role_code: Option<String>,
    #[serde(default)]
    pub tactical_role_origin: String,
    #[serde(default)]
    pub tactical_role_source_position_code: Option<String>,
    #[serde(default = "default_neutral_tactical_role_confidence")]
    pub tactical_role_confidence: f64,
    pub base_ability: f64,
    pub base_ability_confidence: f64,
    pub effective_contribution: f64,
    pub overall_confidence: f64,
    pub expected_minutes_share: f64,
    pub starting_probability: Option<f64>,
    pub components: Vec<ContributionComponent>,
    pub applied_tags: Vec<PlayerDynamicTagRecord>,
    pub calculation_version: String,
}
