use crate::AvailabilityStatus;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    pub metadata: serde_json::Value,
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
