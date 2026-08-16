use chrono::{DateTime, NaiveDate, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct PlayerListRow {
    pub id: Uuid,
    pub canonical_name: String,
    pub localized_name: Option<String>,
    pub alternate_name: Option<String>,
    pub normalized_name: String,
    pub date_of_birth: Option<NaiveDate>,
    pub nationality_code: Option<String>,
    pub preferred_foot: String,
    pub status: String,
    pub current_team_id: Option<Uuid>,
    pub current_team_name: Option<String>,
    pub primary_position_code: Option<String>,
    pub primary_role_code: Option<String>,
    pub position_role_map: Value,
    pub availability_status: Option<String>,
    pub availability_reason: Option<String>,
    pub availability_confidence: Option<f64>,
    pub availability_valid_to: Option<DateTime<Utc>>,
    pub availability_competition_name: Option<String>,
    pub ability_average: Option<f64>,
    pub ability_confidence: Option<f64>,
    pub ability_dimension_count: i32,
}
