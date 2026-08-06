use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPlayerPeriodRecord {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub player_id: Uuid,
    pub player_name: String,
    pub season_id: Option<Uuid>,
    pub season_name: Option<String>,
    pub squad_number: Option<i16>,
    pub valid_from: chrono::NaiveDate,
    pub valid_to: Option<chrono::NaiveDate>,
    pub registration_status: String,
}
