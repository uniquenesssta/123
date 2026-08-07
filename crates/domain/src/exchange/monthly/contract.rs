use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const TEAM_MONTHLY_FORMAT: &str = "football.team-monthly.v1";
pub const PLAYER_MONTHLY_FORMAT: &str = "football.player-monthly.v2";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MonthlyWorkbookKind {
    Team,
    Player,
}

impl MonthlyWorkbookKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Team => "team",
            Self::Player => "player",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyWorkbookExportSummary {
    pub output_path: String,
    pub workbook_kind: MonthlyWorkbookKind,
    pub team_count: u64,
    pub player_count: u64,
    pub coach_count: u64,
    pub related_row_count: u64,
    pub data_gap_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyDataGapRow {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_name: String,
    pub missing_field: String,
    pub last_observed_at: Option<DateTime<Utc>>,
    pub stale_days: Option<i64>,
    pub priority: String,
    pub recommended_action: String,
}
