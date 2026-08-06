use super::default_player_page_limit;
use super::{AvailabilityStatus, PlayerStatus, PreferredFoot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub availability_status: Option<AvailabilityStatus>,
    #[serde(default)]
    pub player_status: Option<PlayerStatus>,
    #[serde(default = "default_player_page_limit")]
    pub limit: u32,
    #[serde(default)]
    pub cursor_name: Option<String>,
    #[serde(default)]
    pub cursor_id: Option<Uuid>,
}
impl Default for PlayerListQuery {
    fn default() -> Self {
        Self {
            search: None,
            team_id: None,
            position_code: None,
            availability_status: None,
            player_status: Some(PlayerStatus::Active),
            limit: default_player_page_limit(),
            cursor_name: None,
            cursor_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerListItem {
    pub id: Uuid,
    pub canonical_name: String,
    pub localized_name: Option<String>,
    pub alternate_name: Option<String>,
    pub normalized_name: String,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub nationality_code: Option<String>,
    pub preferred_foot: PreferredFoot,
    pub status: PlayerStatus,
    pub current_team_id: Option<Uuid>,
    pub current_team_name: Option<String>,
    pub primary_position_code: Option<String>,
    #[serde(default)]
    pub primary_role_code: Option<String>,
    #[serde(default)]
    pub position_role_map: Value,
    pub availability_status: Option<AvailabilityStatus>,
    pub availability_reason: Option<String>,
    pub availability_confidence: Option<f64>,
    pub availability_valid_to: Option<DateTime<Utc>>,
    pub availability_competition_name: Option<String>,
    pub ability_average: Option<f64>,
    pub ability_confidence: Option<f64>,
    pub ability_dimension_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerListPage {
    pub items: Vec<PlayerListItem>,
    pub next_cursor_name: Option<String>,
    pub next_cursor_id: Option<Uuid>,
    pub has_more: bool,
}
