use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPositionDraft {
    pub player_id: Uuid,
    pub position_code: String,
    pub proficiency: f64,
    #[serde(default)]
    pub default_role_code: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub valid_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPositionRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub position_code: String,
    pub position_name: String,
    pub position_group: String,
    pub proficiency: f64,
    #[serde(default)]
    pub default_role_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<chrono::NaiveDate>,
    pub valid_to: Option<chrono::NaiveDate>,
}
