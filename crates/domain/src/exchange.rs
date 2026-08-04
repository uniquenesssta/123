use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const MATCH_LINEUP_IMPORT_FORMAT: &str = "football.match-lineup.v2";
pub const MATCH_LINEUP_IMPORT_LEGACY_FORMAT: &str = "football.match-lineup.v1";
pub const AI_MATCH_PACKAGE_FORMAT: &str = "football.ai-match-package.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDynamicTagDefinitionRecord {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub default_value: f64,
    pub default_ttl_hours: i32,
    pub is_multiplier: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDynamicTagDraft {
    pub player_id: Uuid,
    pub tag_code: String,
    pub value: f64,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub opponent_team_id: Option<Uuid>,
    #[serde(default = "default_sample_size")]
    pub sample_size: i32,
    #[serde(default = "default_source_type")]
    pub source_type: String,
    pub calculation_version: String,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_confidence() -> f64 {
    1.0
}

fn default_sample_size() -> i32 {
    1
}

fn default_source_type() -> String {
    "manual".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDynamicTagRecord {
    pub id: Uuid,
    pub player_id: Uuid,
    pub tag_code: String,
    pub tag_name: String,
    pub category: String,
    pub value: f64,
    pub label: Option<String>,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub position_code: Option<String>,
    pub opponent_team_id: Option<Uuid>,
    pub opponent_team_name: Option<String>,
    pub sample_size: i32,
    pub source_type: String,
    pub calculation_version: String,
    pub metadata: Value,
}

fn default_neutral_tactical_role_confidence() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMatchContributionRequest {
    pub player_id: Uuid,
    #[serde(default)]
    pub match_id: Option<Uuid>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub role_code: Option<String>,
    #[serde(default)]
    pub role_origin: Option<String>,
    #[serde(default)]
    pub role_source_position_code: Option<String>,
    #[serde(default)]
    pub opponent_team_id: Option<Uuid>,
    pub as_of: DateTime<Utc>,
    #[serde(default)]
    pub data_cutoff_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub expected_minutes: Option<i16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributionComponent {
    pub code: String,
    pub label: String,
    pub value: f64,
    pub confidence: f64,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMatchContribution {
    pub player_id: Uuid,
    pub player_name: String,
    pub match_id: Option<Uuid>,
    pub as_of: DateTime<Utc>,
    #[serde(default)]
    pub position_code: Option<String>,
    #[serde(default)]
    pub tactical_role_code: Option<String>,
    #[serde(default)]
    pub tactical_role_origin: String,
    #[serde(default)]
    pub tactical_role_source_position_code: Option<String>,
    #[serde(default = "default_neutral_tactical_role_confidence")]
    pub tactical_role_confidence: f64,
    pub base_ability: f64,
    pub base_ability_confidence: f64,
    pub effective_contribution: f64,
    pub overall_confidence: f64,
    pub expected_minutes_share: f64,
    pub starting_probability: Option<f64>,
    pub components: Vec<ContributionComponent>,
    pub applied_tags: Vec<PlayerDynamicTagRecord>,
    pub calculation_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLineupExportData {
    #[serde(default)]
    pub selected_match: Option<crate::MatchRecord>,
    #[serde(default)]
    pub lineups: Vec<crate::LineupRecord>,
    pub competitions: Vec<crate::CompetitionRecord>,
    pub teams: Vec<crate::TeamOption>,
    pub formations: Vec<crate::FormationRecord>,
    pub coaches: Vec<crate::CoachListItem>,
    pub positions: Vec<crate::PositionReference>,
    pub players: Vec<MatchLineupPlayerReference>,
    #[serde(default)]
    pub dynamic_tag_definitions: Vec<PlayerDynamicTagDefinitionRecord>,
    #[serde(default)]
    pub dynamic_tags: Vec<PlayerDynamicTagRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLineupPlayerReference {
    pub player_id: Uuid,
    pub canonical_name: String,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub current_team_id: Option<Uuid>,
    pub current_team_name: Option<String>,
    pub primary_position_code: Option<String>,
    #[serde(default)]
    pub primary_role_code: Option<String>,
    pub availability_status: Option<crate::AvailabilityStatus>,
    pub ability_average: Option<f64>,
    pub ability_confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchLineupExportSummary {
    pub output_path: String,
    pub match_count: u64,
    pub lineup_count: u64,
    pub player_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMatchPackageManifest {
    pub format_version: String,
    pub created_at: DateTime<Utc>,
    pub match_id: Uuid,
    pub match_key: String,
    pub workbook_file: String,
    pub context_file: String,
    pub instructions_file: String,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMatchPackageContext {
    pub match_record: crate::MatchRecord,
    pub competition: Option<crate::CompetitionRecord>,
    pub lineups: Vec<crate::LineupRecord>,
    pub players: Vec<AiMatchPlayerContext>,
    pub generated_at: DateTime<Utc>,
    pub data_quality: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMatchPlayerContext {
    pub player: crate::PlayerRecord,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    #[serde(default)]
    pub lineup_status: String,
    #[serde(default)]
    pub tactical_role_code: Option<String>,
    #[serde(default)]
    pub tactical_role_origin: String,
    #[serde(default)]
    pub tactical_role_source_position_code: Option<String>,
    #[serde(default)]
    pub lineup_role: Option<String>,
    pub expected_minutes: Option<i16>,
    pub ability_profile: Option<crate::PlayerAbilityProfile>,
    pub availability: Vec<crate::PlayerAvailabilityRecord>,
    pub dynamic_tags: Vec<PlayerDynamicTagRecord>,
    pub contribution: Option<PlayerMatchContribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMatchPackageSummary {
    pub output_path: String,
    pub match_id: Uuid,
    pub match_key: String,
    pub player_count: u64,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparedMatchPredictionInput {
    pub match_record: crate::MatchRecord,
    pub competition_kind: crate::CompetitionKind,
    pub snapshot_type: String,
    pub match_input: Value,
    pub data_quality: Value,
}
