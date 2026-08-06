use super::{CoachNameRecord, CoachRecord, TeamCoachPeriodRecord};
use crate::ExternalEntityIdRecord;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachDetail {
    pub coach: CoachRecord,
    pub names: Vec<CoachNameRecord>,
    pub team_periods: Vec<TeamCoachPeriodRecord>,
    pub external_ids: Vec<ExternalEntityIdRecord>,
}
