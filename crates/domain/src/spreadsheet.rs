use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetRawRow {
    pub sheet_name: String,
    pub row_number: u32,
    pub entity_type: SpreadsheetEntityType,
    pub action: SpreadsheetAction,
    pub values: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetParsedWorkbook {
    pub format_version: String,
    pub source_file_name: String,
    pub source_sha256: String,
    pub rows: Vec<SpreadsheetRawRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetConflictCandidate {
    pub entity_id: Uuid,
    pub display_name: String,
    #[serde(default)]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetImportRow {
    pub id: Uuid,
    pub sheet_name: String,
    pub row_number: u32,
    pub entity_type: SpreadsheetEntityType,
    pub action: SpreadsheetAction,
    pub status: SpreadsheetRowStatus,
    #[serde(default)]
    pub message: Option<String>,
    pub payload: Value,
    #[serde(default)]
    pub matched_entity_id: Option<Uuid>,
    #[serde(default)]
    pub conflict_candidates: Vec<SpreadsheetConflictCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpreadsheetImportCounts {
    pub total: u64,
    pub ready_add: u64,
    pub ready_update: u64,
    pub ready_end_previous: u64,
    pub conflict: u64,
    pub error: u64,
    pub skipped: u64,
    pub imported: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetImportPreview {
    pub batch_id: Uuid,
    pub source_file_name: String,
    pub source_sha256: String,
    pub import_mode: SpreadsheetImportMode,
    pub counts: SpreadsheetImportCounts,
    pub rows: Vec<SpreadsheetImportRow>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetImportResolution {
    pub row_id: Uuid,
    #[serde(default)]
    pub selected_entity_id: Option<Uuid>,
    #[serde(default)]
    pub skip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetImportCommitResult {
    pub batch_id: Uuid,
    pub inserted_count: u64,
    pub updated_count: u64,
    pub ended_previous_count: u64,
    pub skipped_count: u64,
    pub error_count: u64,
    pub finished_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetExportSummary {
    pub output_path: String,
    pub team_count: u64,
    pub player_count: u64,
    pub related_row_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpreadsheetExportData {
    pub teams: Vec<SpreadsheetTeamRow>,
    pub players: Vec<SpreadsheetPlayerRow>,
    pub names: Vec<SpreadsheetPlayerNameRow>,
    pub positions: Vec<SpreadsheetPlayerPositionRow>,
    pub team_periods: Vec<SpreadsheetPlayerTeamPeriodRow>,
    pub abilities: Vec<SpreadsheetPlayerAbilityRow>,
    pub availability: Vec<SpreadsheetPlayerAvailabilityRow>,
    pub dynamic_tags: Vec<SpreadsheetPlayerDynamicTagRow>,
    pub external_ids: Vec<SpreadsheetExternalIdRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetTeamRow {
    pub team_id: Uuid,
    pub canonical_name: String,
    pub country_code: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetPlayerRow {
    pub player_id: Uuid,
    pub canonical_name: String,
    pub date_of_birth: Option<NaiveDate>,
    pub nationality_code: Option<String>,
    pub preferred_foot: String,
    pub height_cm: Option<i16>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetPlayerNameRow {
    pub player_id: Uuid,
    pub player_name: String,
    pub player_birth_date: Option<NaiveDate>,
    pub name: String,
    pub language_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetPlayerPositionRow {
    pub player_id: Uuid,
    pub player_name: String,
    pub player_birth_date: Option<NaiveDate>,
    pub position_code: String,
    pub proficiency: f64,
    #[serde(default)]
    pub default_role_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetPlayerTeamPeriodRow {
    pub player_id: Uuid,
    pub player_name: String,
    pub player_birth_date: Option<NaiveDate>,
    pub team_id: Uuid,
    pub team_name: String,
    pub season_id: Option<Uuid>,
    pub squad_number: Option<i16>,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub registration_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetPlayerAbilityRow {
    pub player_id: Uuid,
    pub player_name: String,
    pub player_birth_date: Option<NaiveDate>,
    pub dimension_code: String,
    pub context_type: String,
    pub context_id: Option<Uuid>,
    pub value: f64,
    pub confidence: f64,
    pub sample_size: i32,
    pub observed_at: DateTime<Utc>,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub calculation_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetPlayerAvailabilityRow {
    pub player_id: Uuid,
    pub player_name: String,
    pub player_birth_date: Option<NaiveDate>,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub status: String,
    pub reason: Option<String>,
    pub confidence: f64,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetPlayerDynamicTagRow {
    pub player_id: Uuid,
    pub player_name: String,
    pub player_birth_date: Option<NaiveDate>,
    pub tag_code: String,
    pub value: f64,
    pub label: Option<String>,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub competition_id: Option<Uuid>,
    pub position_code: Option<String>,
    pub opponent_team_id: Option<Uuid>,
    pub sample_size: i32,
    pub source_type: String,
    pub calculation_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetExternalIdRow {
    pub provider_code: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_name: String,
    pub external_id: String,
}
