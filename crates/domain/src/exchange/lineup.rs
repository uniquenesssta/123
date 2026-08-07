use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const MATCH_LINEUP_IMPORT_FORMAT: &str = "football.match-lineup.v2";
pub const MATCH_LINEUP_IMPORT_LEGACY_FORMAT: &str = "football.match-lineup.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLineupExportData {
    #[serde(default)]
    pub selected_match: Option<crate::MatchRecord>,
    #[serde(default)]
    pub lineups: Vec<crate::LineupRecord>,
    pub competitions: Vec<crate::CompetitionRecord>,
    pub teams: Vec<crate::TeamOption>,
    pub formations: Vec<crate::FormationRecord>,
    pub coaches: Vec<crate::CoachListItem>,
    pub positions: Vec<crate::PositionReference>,
    pub players: Vec<MatchLineupPlayerReference>,
    #[serde(default)]
    pub dynamic_tag_definitions: Vec<super::dynamic_tag::PlayerDynamicTagDefinitionRecord>,
    #[serde(default)]
    pub dynamic_tags: Vec<super::dynamic_tag::PlayerDynamicTagRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLineupPlayerReference {
    pub player_id: Uuid,
    pub canonical_name: String,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub current_team_id: Option<Uuid>,
    pub current_team_name: Option<String>,
    pub primary_position_code: Option<String>,
    #[serde(default)]
    pub primary_role_code: Option<String>,
    pub availability_status: Option<crate::AvailabilityStatus>,
    pub ability_average: Option<f64>,
    pub ability_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLineupExportSummary {
    pub output_path: String,
    pub match_count: u64,
    pub lineup_count: u64,
    pub player_count: u64,
}
