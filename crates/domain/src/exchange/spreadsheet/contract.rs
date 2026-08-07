use serde::{Deserialize, Serialize};

pub const PLAYER_IMPORT_FORMAT: &str = "football.player-import.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpreadsheetImportMode {
    AddOnly,
    AddAndUpdate,
}

impl Default for SpreadsheetImportMode {
    fn default() -> Self {
        Self::AddAndUpdate
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpreadsheetAction {
    Add,
    Upsert,
    Update,
    Clear,
    Skip,
}

impl SpreadsheetAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Upsert => "upsert",
            Self::Update => "update",
            Self::Clear => "clear",
            Self::Skip => "skip",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpreadsheetEntityType {
    Team,
    TeamName,
    Coach,
    CoachName,
    TeamCoachPeriod,
    FormationUsage,
    TeamTacticalObservation,
    TeamAbilityObservation,
    Player,
    PlayerName,
    PlayerPosition,
    PlayerTeamPeriod,
    PlayerAbility,
    PlayerAvailability,
    PlayerDynamicTag,
    ExternalEntityId,
    Match,
    Lineup,
    LineupPlayer,
}

impl SpreadsheetEntityType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Team => "team",
            Self::TeamName => "team_name",
            Self::Coach => "coach",
            Self::CoachName => "coach_name",
            Self::TeamCoachPeriod => "team_coach_period",
            Self::FormationUsage => "formation_usage",
            Self::TeamTacticalObservation => "team_tactical_observation",
            Self::TeamAbilityObservation => "team_ability_observation",
            Self::Player => "player",
            Self::PlayerName => "player_name",
            Self::PlayerPosition => "player_position",
            Self::PlayerTeamPeriod => "player_team_period",
            Self::PlayerAbility => "player_ability",
            Self::PlayerAvailability => "player_availability",
            Self::PlayerDynamicTag => "player_dynamic_tag",
            Self::ExternalEntityId => "external_entity_id",
            Self::Match => "match",
            Self::Lineup => "lineup",
            Self::LineupPlayer => "lineup_player",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpreadsheetRowStatus {
    ReadyAdd,
    ReadyUpdate,
    ReadyEndPrevious,
    Conflict,
    Error,
    Skip,
    Imported,
}

impl SpreadsheetRowStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadyAdd => "ready_add",
            Self::ReadyUpdate => "ready_update",
            Self::ReadyEndPrevious => "ready_end_previous",
            Self::Conflict => "conflict",
            Self::Error => "error",
            Self::Skip => "skip",
            Self::Imported => "imported",
        }
    }

    pub const fn is_ready(self) -> bool {
        matches!(
            self,
            Self::ReadyAdd | Self::ReadyUpdate | Self::ReadyEndPrevious
        )
    }
}
