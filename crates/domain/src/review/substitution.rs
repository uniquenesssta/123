use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstitutionDraft {
    pub team_id: Uuid,
    #[serde(default)] pub player_out_id: Option<Uuid>,
    #[serde(default)] pub player_in_id: Option<Uuid>,
    pub minute: i16,
    #[serde(default = "default_period")] pub period: String,
    #[serde(default)] pub reason: Option<String>,
    #[serde(default)] pub source_document_id: Option<Uuid>,
    #[serde(default)] pub metadata: Value,
}
fn default_period() -> String { "normal_time".to_string() }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubstitutionRecord {
    pub id: Uuid, pub match_id: Uuid, pub team_id: Uuid, pub team_name: String,
    pub player_out_id: Option<Uuid>, pub player_out_name: Option<String>, pub player_in_id: Option<Uuid>,
    pub player_in_name: Option<String>, pub minute: i16, pub period: String, pub reason: Option<String>,
}
