use super::FormationUsageEntryRecord;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationDistributionQuery {
    #[serde(default)]
    pub match_id: Option<Uuid>,
    pub team_id: Uuid,
    #[serde(default)]
    pub coach_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub as_of: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedFormationDistribution {
    pub source_level: String,
    pub source_label: String,
    pub team_id: Uuid,
    pub coach_id: Option<Uuid>,
    pub competition_id: Option<Uuid>,
    pub window_start: Option<chrono::NaiveDate>,
    pub window_end: Option<chrono::NaiveDate>,
    pub observed_matches: i32,
    pub confidence: f64,
    pub entries: Vec<FormationUsageEntryRecord>,
}
