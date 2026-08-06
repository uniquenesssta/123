use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatchCandidate {
    pub id: Uuid,
    pub label: String,
    pub reason: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatchRequest {
    pub entity_type: String,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    #[serde(default)]
    pub provider_id: Option<Uuid>,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub canonical_name: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub nationality_code: Option<String>,
    #[serde(default)]
    pub date_of_birth: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatchResult {
    pub status: String,
    pub matched_id: Option<Uuid>,
    pub candidates: Vec<EntityMatchCandidate>,
}
