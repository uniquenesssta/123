use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::contribution::PlayerMatchContribution;
use super::dynamic_tag::PlayerDynamicTagRecord;

pub const AI_MATCH_PACKAGE_FORMAT: &str = "football.ai-match-package.v1";

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
