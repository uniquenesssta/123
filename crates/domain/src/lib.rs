pub mod coach;
pub mod competition;
pub mod formation;
pub mod player;
pub mod routing;
pub mod shared;
pub mod team;

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
pub use coach::*;
pub use competition::*;
pub use formation::*;
pub use player::*;
pub use routing::*;
pub use shared::*;
pub use team::*;

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

fn default_true() -> bool {
    true
}

fn default_team_page_limit() -> u32 {
    50
}

fn default_confidence() -> f64 {
    1.0
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
