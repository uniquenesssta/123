use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
