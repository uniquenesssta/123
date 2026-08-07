use crate::AvailabilityStatus;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
