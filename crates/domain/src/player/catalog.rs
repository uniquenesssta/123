use super::{PlayerStatus, PreferredFoot};
use crate::{
    AbilityDimensionRecord, DataProviderRecord, FormationRecord, MatchRecord,
    PlayerDynamicTagDefinitionRecord, PositionReference, SeasonTeamMembershipOption, TeamOption,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDraft {
    pub canonical_name: String,
    #[serde(default)]
    pub date_of_birth: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub nationality_code: Option<String>,
    #[serde(default)]
    pub preferred_foot: PreferredFoot,
    #[serde(default)]
    pub height_cm: Option<i16>,
    #[serde(default)]
    pub status: PlayerStatus,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRecord {
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub nationality_code: Option<String>,
    pub preferred_foot: PreferredFoot,
    pub height_cm: Option<i16>,
    pub status: PlayerStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerCatalogReferenceData {
    pub teams: Vec<TeamOption>,
    #[serde(default)]
    pub season_team_memberships: Vec<SeasonTeamMembershipOption>,
    pub formations: Vec<FormationRecord>,
    pub providers: Vec<DataProviderRecord>,
    pub positions: Vec<PositionReference>,
    pub ability_dimensions: Vec<AbilityDimensionRecord>,
    pub dynamic_tag_definitions: Vec<PlayerDynamicTagDefinitionRecord>,
    pub upcoming_matches: Vec<MatchRecord>,
    pub managed_matches: Vec<MatchRecord>,
}
