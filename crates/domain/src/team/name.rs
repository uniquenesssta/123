use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamNameDraft {
    pub team_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub language_code: Option<String>,
    #[serde(default)]
    pub valid_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamNameRecord {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub language_code: Option<String>,
    pub valid_from: Option<chrono::NaiveDate>,
    pub valid_to: Option<chrono::NaiveDate>,
}
