use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDynamicTagDefinitionRecord {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub default_value: f64,
    pub default_ttl_hours: i32,
    pub is_multiplier: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDynamicTagDraft {
    pub player_id: Uuid,
    pub tag_code: String,
    pub value: f64,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub opponent_team_id: Option<Uuid>,
    #[serde(default = "default_sample_size")]
    pub sample_size: i32,
    #[serde(default = "default_source_type")]
    pub source_type: String,
    pub calculation_version: String,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_confidence() -> f64 {
    1.0
}

fn default_sample_size() -> i32 {
    1
}

fn default_source_type() -> String {
    "manual".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDynamicTagRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub tag_code: String,
    pub tag_name: String,
    pub category: String,
    pub value: f64,
    pub label: Option<String>,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub position_code: Option<String>,
    pub opponent_team_id: Option<Uuid>,
    pub opponent_team_name: Option<String>,
    pub sample_size: i32,
    pub source_type: String,
    pub calculation_version: String,
    pub metadata: Value,
}
