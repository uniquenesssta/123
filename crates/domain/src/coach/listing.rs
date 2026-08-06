use crate::default_team_page_limit;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub active_only: bool,
    #[serde(default = "default_team_page_limit")]
    pub limit: u32,
}
impl Default for CoachListQuery {
    fn default() -> Self {
        Self {
            search: None,
            active_only: true,
            limit: default_team_page_limit(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachListItem {
    pub id: Uuid,
    pub canonical_name: String,
    pub nationality_code: Option<String>,
    pub status: String,
    pub current_team_id: Option<Uuid>,
    pub current_team_name: Option<String>,
    pub current_role: Option<String>,
}
