use crate::{LineupRecord, MatchRecord};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const FORMAL_LINEUP_SNAPSHOT_TYPES: [&str; 4] = ["T-N", "T-24h", "T-6h", "T-1h"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLineupTeamChain {
    pub team_id: Uuid,
    pub team_name: String,
    pub team_side: String,
    pub selected_lineup_id: Option<Uuid>,
    pub versions: Vec<LineupRecord>,
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLineupChain {
    pub match_record: MatchRecord,
    pub snapshot_type: String,
    pub data_window_start_time: Option<DateTime<Utc>>,
    pub data_cutoff_time: DateTime<Utc>,
    pub home: MatchLineupTeamChain,
    pub away: MatchLineupTeamChain,
    pub ready_for_model: bool,
    pub blocking_issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMatchLineupHistoryItem {
    pub match_id: Uuid,
    pub match_key: String,
    pub opponent_team_id: Uuid,
    pub opponent_team_name: String,
    pub venue_side: String,
    pub kickoff_time: DateTime<Utc>,
    pub lineup: LineupRecord,
}
