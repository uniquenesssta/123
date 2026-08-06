use super::{TeamNameRecord, TeamPlayerPeriodRecord, TeamProfileRecord, TeamRecord};
use crate::{
    AvailabilityStatus, FormationUsageDistributionRecord, MatchStatus,
    ResolvedFormationDistribution, TeamCoachPeriodRecord,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDetail {
    pub team: TeamRecord,
    pub names: Vec<TeamNameRecord>,
    pub profile: Option<TeamProfileRecord>,
    pub squad: Vec<TeamSquadPlayer>,
    pub player_periods: Vec<TeamPlayerPeriodRecord>,
    pub coach_periods: Vec<TeamCoachPeriodRecord>,
    pub recent_matches: Vec<TeamRecentMatch>,
    pub formation_usage: Vec<FormationUsageDistributionRecord>,
    pub resolved_formation_distribution: ResolvedFormationDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamSquadPlayer {
    pub player_id: Uuid,
    pub player_name: String,
    pub localized_name: Option<String>,
    pub position_code: Option<String>,
    #[serde(default)]
    pub role_code: Option<String>,
    pub squad_number: Option<i16>,
    pub registration_status: String,
    pub availability_status: Option<AvailabilityStatus>,
    pub ability_average: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRecentMatch {
    pub match_id: Uuid,
    pub opponent_team_id: Uuid,
    pub opponent_team_name: String,
    pub kickoff_time: DateTime<Utc>,
    pub venue_side: String,
    pub status: MatchStatus,
    pub goals_for: Option<i16>,
    pub goals_against: Option<i16>,
}
