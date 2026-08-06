pub mod competition;
pub mod routing;

mod analytics;
mod api_workspace;
mod exchange;
mod fact_pipeline;
mod lineup_chain;
mod match_event;
mod match_review_package;
mod match_review_workflow;
mod monthly_workbook;
mod p4_orchestration;
mod p4_persistence;
mod p4_workbench;
mod postmatch;
mod prediction_readiness;
mod release_acceptance;
mod research_gateway;
mod review;
mod spreadsheet;
mod team_package;
pub use competition::*;
pub use routing::*;

pub use analytics::*;
pub use api_workspace::*;
pub use exchange::*;
pub use fact_pipeline::*;
pub use lineup_chain::*;
pub use match_event::*;
pub use match_review_package::*;
pub use match_review_workflow::*;
pub use monthly_workbook::*;
pub use p4_orchestration::*;
pub use p4_persistence::*;
pub use p4_workbench::*;
pub use postmatch::*;
pub use prediction_readiness::*;
pub use release_acceptance::*;
pub use research_gateway::*;
pub use review::*;
pub use spreadsheet::*;
pub use team_package::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchContext {
    pub match_key: String,
    pub kickoff_time: DateTime<Utc>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    #[serde(default)]
    pub competition_kind: CompetitionKind,
    pub home_team_name: String,
    pub away_team_name: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionSummary {
    pub home_win: f64,
    pub draw: f64,
    pub away_win: f64,
    pub btts: Option<f64>,
    pub over_2_5: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedModelRun {
    pub id: Uuid,
    pub match_key: String,
    pub identity: ModelIdentity,
    pub snapshot_type: String,
    pub created_at: DateTime<Utc>,
    pub summary: PredictionSummary,
    pub output: Value,
}

// ─────────────────────────────────────────────────────────────────────────────
// 第三阶段：球员目录、球队归属、可用性、能力观察与阵容。
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PreferredFoot {
    Left,
    Right,
    Both,
    Unknown,
}

impl PreferredFoot {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Both => "both",
            Self::Unknown => "unknown",
        }
    }
}

impl Default for PreferredFoot {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
    Active,
    Inactive,
    Retired,
    Unknown,
}

impl PlayerStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Retired => "retired",
            Self::Unknown => "unknown",
        }
    }
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    Available,
    Doubtful,
    Unavailable,
    Injured,
    Suspended,
    Rested,
    Returning,
    Unknown,
}

impl AvailabilityStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Doubtful => "doubtful",
            Self::Unavailable => "unavailable",
            Self::Injured => "injured",
            Self::Suspended => "suspended",
            Self::Rested => "rested",
            Self::Returning => "returning",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    Scheduled,
    Live,
    Finished,
    Postponed,
    Cancelled,
}

impl MatchStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Live => "live",
            Self::Finished => "finished",
            Self::Postponed => "postponed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl Default for MatchStatus {
    fn default() -> Self {
        Self::Scheduled
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LineupType {
    Expected,
    Confirmed,
    Actual,
}

impl LineupType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expected => "expected",
            Self::Confirmed => "confirmed",
            Self::Actual => "actual",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDraft {
    pub canonical_name: String,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRecord {
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub country_code: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamOption {
    pub id: Uuid,
    pub canonical_name: String,
    pub country_code: Option<String>,
    #[serde(default = "default_team_type")]
    pub team_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub team_type: Option<String>,
    #[serde(default = "default_true")]
    pub active_only: bool,
    #[serde(default = "default_team_page_limit")]
    pub limit: u32,
    #[serde(default)]
    pub cursor_name: Option<String>,
    #[serde(default)]
    pub cursor_id: Option<Uuid>,
}

fn default_true() -> bool {
    true
}

fn default_team_page_limit() -> u32 {
    50
}

impl Default for TeamListQuery {
    fn default() -> Self {
        Self {
            search: None,
            country_code: None,
            team_type: None,
            active_only: true,
            limit: default_team_page_limit(),
            cursor_name: None,
            cursor_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamListItem {
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub country_code: Option<String>,
    pub team_type: String,
    pub current_coach_name: Option<String>,
    pub is_active: bool,
    pub current_player_count: i64,
    pub unavailable_player_count: i64,
    pub squad_ability_average: Option<f64>,
    pub profile_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamListPage {
    pub items: Vec<TeamListItem>,
    pub next_cursor_name: Option<String>,
    pub next_cursor_id: Option<Uuid>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamNameDraft {
    pub team_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub language_code: Option<String>,
    #[serde(default)]
    pub valid_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamNameRecord {
    pub id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub language_code: Option<String>,
    pub valid_from: Option<chrono::NaiveDate>,
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamProfileDraft {
    #[serde(default)]
    pub short_name: Option<String>,
    #[serde(default = "default_team_type")]
    pub team_type: String,
    #[serde(default)]
    pub founded_year: Option<i16>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub stadium: Option<String>,
    #[serde(default)]
    pub head_coach: Option<String>,
    #[serde(default)]
    pub default_formation: Option<String>,
    #[serde(default = "default_tactical_style")]
    pub tactical_style: String,
    #[serde(default)]
    pub attack_rating: Option<f64>,
    #[serde(default)]
    pub midfield_rating: Option<f64>,
    #[serde(default)]
    pub defence_rating: Option<f64>,
    #[serde(default)]
    pub goalkeeper_rating: Option<f64>,
    #[serde(default)]
    pub reputation: Option<f64>,
    #[serde(default = "default_team_profile_confidence")]
    pub data_confidence: f64,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_team_type() -> String {
    "club".to_string()
}

fn default_tactical_style() -> String {
    "balanced".to_string()
}

fn default_team_profile_confidence() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamProfileRecord {
    pub team_id: Uuid,
    pub short_name: Option<String>,
    pub team_type: String,
    pub founded_year: Option<i16>,
    pub city: Option<String>,
    pub stadium: Option<String>,
    pub head_coach: Option<String>,
    pub default_formation: Option<String>,
    pub tactical_style: String,
    pub attack_rating: Option<f64>,
    pub midfield_rating: Option<f64>,
    pub defence_rating: Option<f64>,
    pub goalkeeper_rating: Option<f64>,
    pub reputation: Option<f64>,
    pub data_confidence: f64,
    pub notes: Option<String>,
    pub metadata: Value,
    pub updated_at: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationRecord {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub line_structure: String,
    pub slot_definition: Value,
    pub is_builtin: bool,
    pub is_active: bool,
    pub sort_order: i16,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageEntryDraft {
    pub formation_id: Uuid,
    pub usage_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageDistributionDraft {
    pub scope_type: String,
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub coach_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default = "default_formation_window_preset")]
    pub window_preset: String,
    #[serde(default)]
    pub window_start: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub window_end: Option<chrono::NaiveDate>,
    pub observed_matches: i32,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "default_formation_alpha")]
    pub alpha: f64,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
    pub entries: Vec<FormationUsageEntryDraft>,
}

fn default_formation_window_preset() -> String {
    "custom".to_string()
}

fn default_formation_alpha() -> f64 {
    3.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageEntryRecord {
    pub id: Uuid,
    pub formation_id: Uuid,
    pub formation_code: String,
    pub formation_name: String,
    pub usage_count: i32,
    pub raw_probability: f64,
    pub smoothed_probability: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageDistributionRecord {
    pub scope_type: String,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub coach_id: Option<Uuid>,
    pub coach_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub window_preset: String,
    pub window_start: chrono::NaiveDate,
    pub window_end: chrono::NaiveDate,
    pub observed_matches: i32,
    pub confidence: f64,
    pub alpha: f64,
    pub observed_at: DateTime<Utc>,
    pub entries: Vec<FormationUsageEntryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationUsageListQuery {
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub coach_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default = "default_formation_usage_limit")]
    pub limit: u32,
}

fn default_formation_usage_limit() -> u32 {
    100
}

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
pub struct CoachDraft {
    pub canonical_name: String,
    #[serde(default)]
    pub nationality_code: Option<String>,
    #[serde(default = "default_coach_status")]
    pub status: String,
    #[serde(default)]
    pub metadata: Value,
}

fn default_coach_status() -> String {
    "active".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachRecord {
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub nationality_code: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub active_only: bool,
    #[serde(default = "default_team_page_limit")]
    pub limit: u32,
}

impl Default for CoachListQuery {
    fn default() -> Self {
        Self {
            search: None,
            active_only: true,
            limit: default_team_page_limit(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachListItem {
    pub id: Uuid,
    pub canonical_name: String,
    pub nationality_code: Option<String>,
    pub status: String,
    pub current_team_id: Option<Uuid>,
    pub current_team_name: Option<String>,
    pub current_role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachNameDraft {
    pub coach_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub language_code: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub valid_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachNameRecord {
    pub id: Uuid,
    pub coach_id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub language_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<chrono::NaiveDate>,
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCoachPeriodDraft {
    pub team_id: Uuid,
    pub coach_id: Uuid,
    #[serde(default = "default_coach_role")]
    pub role: String,
    pub valid_from: chrono::NaiveDate,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub is_interim: bool,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default = "default_true")]
    pub end_previous: bool,
    #[serde(default)]
    pub metadata: Value,
}

fn default_coach_role() -> String {
    "head_coach".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamCoachPeriodRecord {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub coach_id: Uuid,
    pub coach_name: String,
    pub role: String,
    pub valid_from: chrono::NaiveDate,
    pub valid_to: Option<chrono::NaiveDate>,
    pub is_interim: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamPlayerPeriodRecord {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub player_id: Uuid,
    pub player_name: String,
    pub season_id: Option<Uuid>,
    pub season_name: Option<String>,
    pub squad_number: Option<i16>,
    pub valid_from: chrono::NaiveDate,
    pub valid_to: Option<chrono::NaiveDate>,
    pub registration_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachDetail {
    pub coach: CoachRecord,
    pub names: Vec<CoachNameRecord>,
    pub team_periods: Vec<TeamCoachPeriodRecord>,
    pub external_ids: Vec<ExternalEntityIdRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReferenceQuery {
    pub entity_type: String,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default = "default_true")]
    pub active_only: bool,
    #[serde(default = "default_team_page_limit")]
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReferenceRecord {
    pub entity_type: String,
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub country_code: Option<String>,
    pub nationality_code: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub status: String,
    pub aliases: Vec<String>,
    pub external_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatchRequest {
    pub entity_type: String,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    #[serde(default)]
    pub provider_id: Option<Uuid>,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub canonical_name: Option<String>,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub nationality_code: Option<String>,
    #[serde(default)]
    pub date_of_birth: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatchCandidate {
    pub id: Uuid,
    pub label: String,
    pub reason: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMatchResult {
    pub status: String,
    pub matched_id: Option<Uuid>,
    pub candidates: Vec<EntityMatchCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReferenceCount {
    pub relation: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDeletionCheck {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub label: String,
    pub exists: bool,
    pub can_permanently_delete: bool,
    pub must_archive: bool,
    pub references: Vec<EntityReferenceCount>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkArchiveFailedItem {
    pub id: Uuid,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkArchiveResult {
    pub entity_type: String,
    pub requested_count: u64,
    pub archived_ids: Vec<Uuid>,
    pub already_archived_ids: Vec<Uuid>,
    pub failed: Vec<BulkArchiveFailedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDeleteBlockedItem {
    pub id: Uuid,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDeleteResult {
    pub requested_count: u64,
    pub deleted_ids: Vec<Uuid>,
    pub blocked: Vec<BulkDeleteBlockedItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamForceDeleteRequest {
    pub team_id: Uuid,
    pub confirmation_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamForceDeletePreview {
    pub team_id: Uuid,
    pub label: String,
    pub confirmation_text: String,
    pub total_rows: u64,
    pub references: Vec<EntityReferenceCount>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamForceDeleteResult {
    pub team_id: Uuid,
    pub label: String,
    pub deleted_match_ids: Vec<Uuid>,
    pub deleted_player_ids: Vec<Uuid>,
    pub deleted_coach_ids: Vec<Uuid>,
    pub deleted_import_batch_ids: Vec<Uuid>,
    pub deleted_counts: std::collections::BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProviderDraft {
    pub code: String,
    pub name: String,
    pub provider_type: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProviderRecord {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: Option<String>,
    pub is_active: bool,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub availability_status: Option<AvailabilityStatus>,
    #[serde(default)]
    pub player_status: Option<PlayerStatus>,
    #[serde(default = "default_player_page_limit")]
    pub limit: u32,
    #[serde(default)]
    pub cursor_name: Option<String>,
    #[serde(default)]
    pub cursor_id: Option<Uuid>,
}

fn default_player_page_limit() -> u32 {
    50
}

impl Default for PlayerListQuery {
    fn default() -> Self {
        Self {
            search: None,
            team_id: None,
            position_code: None,
            availability_status: None,
            player_status: Some(PlayerStatus::Active),
            limit: default_player_page_limit(),
            cursor_name: None,
            cursor_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerListItem {
    pub id: Uuid,
    pub canonical_name: String,
    pub localized_name: Option<String>,
    pub alternate_name: Option<String>,
    pub normalized_name: String,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub nationality_code: Option<String>,
    pub preferred_foot: PreferredFoot,
    pub status: PlayerStatus,
    pub current_team_id: Option<Uuid>,
    pub current_team_name: Option<String>,
    pub primary_position_code: Option<String>,
    #[serde(default)]
    pub primary_role_code: Option<String>,
    #[serde(default)]
    pub position_role_map: Value,
    pub availability_status: Option<AvailabilityStatus>,
    pub availability_reason: Option<String>,
    pub availability_confidence: Option<f64>,
    pub availability_valid_to: Option<DateTime<Utc>>,
    pub availability_competition_name: Option<String>,
    pub ability_average: Option<f64>,
    pub ability_confidence: Option<f64>,
    pub ability_dimension_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerListPage {
    pub items: Vec<PlayerListItem>,
    pub next_cursor_name: Option<String>,
    pub next_cursor_id: Option<Uuid>,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerNameDraft {
    pub player_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub language_code: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub valid_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerNameRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub language_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<chrono::NaiveDate>,
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPositionDraft {
    pub player_id: Uuid,
    pub position_code: String,
    pub proficiency: f64,
    #[serde(default)]
    pub default_role_code: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub valid_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPositionRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub position_code: String,
    pub position_name: String,
    pub position_group: String,
    pub proficiency: f64,
    #[serde(default)]
    pub default_role_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<chrono::NaiveDate>,
    pub valid_to: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTeamPeriodDraft {
    pub player_id: Uuid,
    pub team_id: Uuid,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub squad_number: Option<i16>,
    pub valid_from: chrono::NaiveDate,
    #[serde(default)]
    pub valid_to: Option<chrono::NaiveDate>,
    #[serde(default = "default_registration_status")]
    pub registration_status: String,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
}

fn default_registration_status() -> String {
    "registered".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerTeamPeriodRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub season_id: Option<Uuid>,
    pub season_name: Option<String>,
    pub squad_number: Option<i16>,
    pub valid_from: chrono::NaiveDate,
    pub valid_to: Option<chrono::NaiveDate>,
    pub registration_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAvailabilityDraft {
    pub player_id: Uuid,
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    pub status: AvailabilityStatus,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    pub valid_from: DateTime<Utc>,
    #[serde(default)]
    pub valid_to: Option<DateTime<Utc>>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_confidence() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAvailabilityRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub status: AvailabilityStatus,
    pub reason: Option<String>,
    pub confidence: f64,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityDimensionRecord {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAbilityObservationDraft {
    pub player_id: Uuid,
    pub dimension_code: String,
    #[serde(default = "default_context_type")]
    pub context_type: String,
    #[serde(default)]
    pub context_id: Option<Uuid>,
    pub value: f64,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "default_sample_size")]
    pub sample_size: i32,
    pub observed_at: DateTime<Utc>,
    pub effective_from: DateTime<Utc>,
    #[serde(default)]
    pub effective_to: Option<DateTime<Utc>>,
    pub calculation_version: String,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_context_type() -> String {
    "general".to_string()
}

fn default_sample_size() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAbilityObservationRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub dimension_code: String,
    pub dimension_name: String,
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
pub struct PlayerAbilityProfile {
    pub player_id: Uuid,
    pub abilities: Value,
    pub average_value: Option<f64>,
    pub average_confidence: Option<f64>,
    pub dimension_count: i32,
    pub latest_observed_at: Option<DateTime<Utc>>,
    pub next_expiry_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntityIdDraft {
    pub provider_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub external_id: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntityIdRecord {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub external_id: String,
    pub metadata: Value,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchDraft {
    #[serde(default)]
    pub external_key: String,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    #[serde(default)]
    pub round_id: Option<Uuid>,
    pub home_team_id: Uuid,
    pub away_team_id: Uuid,
    pub kickoff_time: DateTime<Utc>,
    #[serde(default)]
    pub status: MatchStatus,
    #[serde(default)]
    pub venue: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRecord {
    pub id: Uuid,
    pub external_key: String,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub round_id: Option<Uuid>,
    pub home_team_id: Uuid,
    pub home_team_name: String,
    pub away_team_id: Uuid,
    pub away_team_name: String,
    pub kickoff_time: DateTime<Utc>,
    pub status: MatchStatus,
    pub venue: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupPlayerDraft {
    pub player_id: Uuid,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub role_code: Option<String>,
    pub is_starter: bool,
    #[serde(default)]
    pub shirt_number: Option<i16>,
    #[serde(default)]
    pub expected_minutes: Option<i16>,
    #[serde(default)]
    pub actual_minutes: Option<i16>,
    #[serde(default)]
    pub sequence_no: i16,
    #[serde(default)]
    pub bench_order: Option<i16>,
    #[serde(default)]
    pub availability_status: Option<AvailabilityStatus>,
    #[serde(default)]
    pub starting_probability: Option<f64>,
    #[serde(default)]
    pub membership_override: bool,
    #[serde(default)]
    pub source_urls: Vec<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupDraft {
    pub match_id: Uuid,
    pub team_id: Uuid,
    pub lineup_type: LineupType,
    #[serde(default = "default_lineup_snapshot_type")]
    pub snapshot_type: String,
    #[serde(default)]
    pub formation: Option<String>,
    #[serde(default)]
    pub formation_id: Option<Uuid>,
    #[serde(default)]
    pub coach_id: Option<Uuid>,
    pub captured_at: DateTime<Utc>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub source_urls: Vec<String>,
    #[serde(default)]
    pub quality_score: Option<f64>,
    #[serde(default)]
    pub metadata: Value,
    pub players: Vec<LineupPlayerDraft>,
}

fn default_lineup_snapshot_type() -> String {
    "T-1h".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupPairDraft {
    pub home: LineupDraft,
    pub away: LineupDraft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupPairRecord {
    pub home: LineupRecord,
    pub away: LineupRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupPlayerRecord {
    pub player_id: Uuid,
    pub player_name: String,
    pub position_code: Option<String>,
    pub role_code: Option<String>,
    #[serde(default)]
    pub role_origin: String,
    #[serde(default)]
    pub role_source_position_code: Option<String>,
    pub is_starter: bool,
    pub shirt_number: Option<i16>,
    pub expected_minutes: Option<i16>,
    pub actual_minutes: Option<i16>,
    pub sequence_no: i16,
    pub bench_order: Option<i16>,
    pub availability_status: Option<AvailabilityStatus>,
    pub starting_probability: Option<f64>,
    pub membership_override: bool,
    pub source_urls: Vec<String>,
    pub validation_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub team_id: Uuid,
    pub team_name: String,
    pub lineup_type: LineupType,
    pub snapshot_type: String,
    pub formation: Option<String>,
    pub formation_id: Option<Uuid>,
    pub formation_code: Option<String>,
    pub formation_name: Option<String>,
    pub coach_id: Option<Uuid>,
    pub coach_name: Option<String>,
    pub captured_at: DateTime<Utc>,
    pub status: String,
    pub quality_score: Option<f64>,
    pub source_urls: Vec<String>,
    pub supersedes_lineup_id: Option<Uuid>,
    pub model_validation_status: String,
    pub model_eligible: bool,
    pub validation_errors: Vec<String>,
    pub validation_warnings: Vec<String>,
    pub player_count: i64,
    pub starter_count: i64,
    pub players: Vec<LineupPlayerRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamLineupPresetMemberDraft {
    pub player_id: Uuid,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub role_code: Option<String>,
    pub is_starter: bool,
    #[serde(default)]
    pub shirt_number: Option<i16>,
    #[serde(default)]
    pub expected_minutes: Option<i16>,
    #[serde(default)]
    pub sequence_no: i16,
    #[serde(default)]
    pub bench_order: Option<i16>,
    #[serde(default)]
    pub is_captain: bool,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamLineupPresetDraft {
    #[serde(default)]
    pub id: Option<Uuid>,
    pub team_id: Uuid,
    pub name: String,
    #[serde(default)]
    pub formation_id: Option<Uuid>,
    #[serde(default)]
    pub coach_id: Option<Uuid>,
    #[serde(default = "default_lineup_preset_context")]
    pub usage_context: String,
    #[serde(default)]
    pub usage_probability: Option<f64>,
    #[serde(default)]
    pub is_default: bool,
    #[serde(default)]
    pub source_lineup_id: Option<Uuid>,
    #[serde(default)]
    pub notes: Option<String>,
    pub members: Vec<TeamLineupPresetMemberDraft>,
}

fn default_lineup_preset_context() -> String {
    "general".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamLineupPresetMemberRecord {
    pub player_id: Uuid,
    pub player_name: String,
    pub alternate_name: Option<String>,
    pub position_code: Option<String>,
    pub role_code: Option<String>,
    #[serde(default)]
    pub role_origin: String,
    #[serde(default)]
    pub role_source_position_code: Option<String>,
    pub is_starter: bool,
    pub shirt_number: Option<i16>,
    pub expected_minutes: Option<i16>,
    pub sequence_no: i16,
    pub bench_order: Option<i16>,
    pub is_captain: bool,
    pub current_team_id: Option<Uuid>,
    pub current_team_name: Option<String>,
    pub player_status: String,
    pub availability_status: Option<AvailabilityStatus>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamLineupPresetRecord {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub name: String,
    pub formation_id: Option<Uuid>,
    pub formation_code: Option<String>,
    pub formation_name: Option<String>,
    pub coach_id: Option<Uuid>,
    pub coach_name: Option<String>,
    pub usage_context: String,
    pub usage_probability: Option<f64>,
    pub is_default: bool,
    pub status: String,
    pub version: i32,
    pub source_lineup_id: Option<Uuid>,
    pub notes: Option<String>,
    pub starter_count: i64,
    pub member_count: i64,
    pub members: Vec<TeamLineupPresetMemberRecord>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamLineupPresetApplicationPreview {
    pub preset: TeamLineupPresetRecord,
    pub can_apply: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineupHistoryRemovalResult {
    pub lineup_id: Uuid,
    pub removal_mode: String,
    pub restored_lineup_id: Option<Uuid>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionReference {
    pub code: String,
    pub name: String,
    pub position_group: String,
    pub sort_order: i16,
}
