use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4PlanningMatchContext {
    pub match_id: Uuid,
    pub match_key: String,
    pub kickoff_at: DateTime<Utc>,
    pub competition_id: Option<Uuid>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub competition_kind: crate::CompetitionKind,
    pub home_team_name: String,
    pub away_team_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanP4HorizonsCommand {
    pub match_id: Uuid,
    pub explicit_rule_package_id: Uuid,
    #[serde(default)]
    pub requested_fact_keys: Vec<String>,
}
