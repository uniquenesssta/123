use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachNameDraft {
    pub coach_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub language_code: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub valid_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachNameRecord {
    pub id: Uuid,
    pub coach_id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub language_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<chrono::NaiveDate>,
    pub valid_to: Option<chrono::NaiveDate>,
}
