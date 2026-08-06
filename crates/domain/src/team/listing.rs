use crate::{default_team_page_limit, default_true};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub team_type: Option<String>,
    #[serde(default = "default_true")]
    pub active_only: bool,
    #[serde(default = "default_team_page_limit")]
    pub limit: u32,
    #[serde(default)]
    pub cursor_name: Option<String>,
    #[serde(default)]
    pub cursor_id: Option<Uuid>,
}
impl Default for TeamListQuery {
    fn default() -> Self {
        Self {
            search: None,
            country_code: None,
            team_type: None,
            active_only: true,
            limit: default_team_page_limit(),
            cursor_name: None,
            cursor_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamListItem {
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub country_code: Option<String>,
    pub team_type: String,
    pub current_coach_name: Option<String>,
    pub is_active: bool,
    pub current_player_count: i64,
    pub unavailable_player_count: i64,
    pub squad_ability_average: Option<f64>,
    pub profile_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamListPage {
    pub items: Vec<TeamListItem>,
    pub next_cursor_name: Option<String>,
    pub next_cursor_id: Option<Uuid>,
    pub has_more: bool,
}
