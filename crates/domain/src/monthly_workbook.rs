use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const TEAM_MONTHLY_FORMAT: &str = "football.team-monthly.v1";
pub const PLAYER_MONTHLY_FORMAT: &str = "football.player-monthly.v2";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MonthlyWorkbookKind {
    Team,
    Player,
}

impl MonthlyWorkbookKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Team => "team",
            Self::Player => "player",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyWorkbookExportSummary {
    pub output_path: String,
    pub workbook_kind: MonthlyWorkbookKind,
    pub team_count: u64,
    pub player_count: u64,
    pub coach_count: u64,
    pub related_row_count: u64,
    pub data_gap_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyDataGapRow {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_name: String,
    pub missing_field: String,
    pub last_observed_at: Option<DateTime<Utc>>,
    pub stale_days: Option<i64>,
    pub priority: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TeamMonthlyWorkbookData {
    pub teams: Vec<TeamMonthlyTeamRow>,
    pub names: Vec<TeamMonthlyNameRow>,
    pub coaches: Vec<TeamMonthlyCoachRow>,
    pub coach_periods: Vec<TeamMonthlyCoachPeriodRow>,
    pub formation_usage: Vec<TeamMonthlyFormationUsageRow>,
    pub tactical_observations: Vec<TeamTacticalObservationRow>,
    pub ability_observations: Vec<TeamAbilityObservationRow>,
    pub data_gaps: Vec<MonthlyDataGapRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMonthlyTeamRow {
    pub team_id: Uuid,
    pub official_name: String,
    pub short_name: Option<String>,
    pub team_type: String,
    pub country_code: Option<String>,
    pub city: Option<String>,
    pub founded_year: Option<i16>,
    pub stadium: Option<String>,
    pub is_active: bool,
    pub profile_observed_at: Option<DateTime<Utc>>,
    pub data_confidence: f64,
    pub notes: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMonthlyNameRow {
    pub team_id: Uuid,
    pub official_name: String,
    pub name_value: String,
    pub language_code: Option<String>,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMonthlyCoachRow {
    pub coach_id: Uuid,
    pub official_name: String,
    pub nationality_code: Option<String>,
    pub status: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMonthlyCoachPeriodRow {
    pub team_id: Uuid,
    pub team_name: String,
    pub coach_id: Uuid,
    pub coach_name: String,
    pub role: String,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub is_interim: bool,
    pub confidence: f64,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMonthlyFormationUsageRow {
    pub scope_type: String,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub coach_id: Option<Uuid>,
    pub coach_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub formation_id: Uuid,
    pub formation_code: String,
    pub window_preset: String,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub observed_matches: i32,
    pub usage_count: i32,
    pub raw_probability: f64,
    pub smoothed_probability: f64,
    pub confidence: f64,
    pub alpha: f64,
    pub observed_at: DateTime<Utc>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTacticalObservationRow {
    pub team_id: Uuid,
    pub team_name: String,
    pub coach_id: Option<Uuid>,
    pub coach_name: Option<String>,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub build_up_style: Option<String>,
    pub progression_style: Option<String>,
    pub attacking_width: Option<String>,
    pub pressing_intensity: Option<String>,
    pub defensive_block: Option<String>,
    pub transition_speed: Option<String>,
    pub set_piece_tendency: Option<String>,
    pub tactical_summary: Option<String>,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAbilityObservationRow {
    pub team_id: Uuid,
    pub team_name: String,
    pub observed_at: DateTime<Utc>,
    pub window_start: NaiveDate,
    pub window_end: NaiveDate,
    pub attack_rating: Option<f64>,
    pub midfield_rating: Option<f64>,
    pub defence_rating: Option<f64>,
    pub goalkeeper_rating: Option<f64>,
    pub squad_depth_rating: Option<f64>,
    pub stability_rating: Option<f64>,
    pub sample_size: i32,
    pub methodology: Option<String>,
    pub confidence: f64,
    pub metadata: Value,
}
