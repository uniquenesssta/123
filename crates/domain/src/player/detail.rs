use super::{
    PlayerAbilityObservationRecord, PlayerAbilityProfile, PlayerAvailabilityRecord,
    PlayerNameRecord, PlayerPositionRecord, PlayerRecord, PlayerTeamPeriodRecord,
};
use crate::{ExternalEntityIdRecord, PlayerDynamicTagRecord};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDetail {
    pub player: PlayerRecord,
    pub names: Vec<PlayerNameRecord>,
    pub positions: Vec<PlayerPositionRecord>,
    pub team_periods: Vec<PlayerTeamPeriodRecord>,
    pub availability: Vec<PlayerAvailabilityRecord>,
    pub ability_profile: Option<PlayerAbilityProfile>,
    pub ability_observations: Vec<PlayerAbilityObservationRecord>,
    pub dynamic_tags: Vec<PlayerDynamicTagRecord>,
    pub external_ids: Vec<ExternalEntityIdRecord>,
}
