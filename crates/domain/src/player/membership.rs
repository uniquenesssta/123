use super::default_registration_status;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTeamPeriodDraft {
    pub player_id: Uuid,
    pub team_id: Uuid,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub squad_number: Option<i16>,
    pub valid_from: chrono::NaiveDate,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
    #[serde(default = "default_registration_status")]
    pub registration_status: String,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTeamPeriodRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub season_id: Option<Uuid>,
    pub season_name: Option<String>,
    pub squad_number: Option<i16>,
    pub valid_from: chrono::NaiveDate,
    pub valid_to: Option<chrono::NaiveDate>,
    pub registration_status: String,
}
