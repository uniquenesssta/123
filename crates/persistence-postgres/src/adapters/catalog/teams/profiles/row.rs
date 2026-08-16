use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct TeamProfileRow {
    pub(super) team_id: Uuid,
    pub(super) short_name: Option<String>,
    pub(super) team_type: String,
    pub(super) founded_year: Option<i16>,
    pub(super) city: Option<String>,
    pub(super) stadium: Option<String>,
    pub(super) head_coach: Option<String>,
    pub(super) default_formation: Option<String>,
    pub(super) tactical_style: String,
    pub(super) attack_rating: Option<f64>,
    pub(super) midfield_rating: Option<f64>,
    pub(super) defence_rating: Option<f64>,
    pub(super) goalkeeper_rating: Option<f64>,
    pub(super) reputation: Option<f64>,
    pub(super) data_confidence: f64,
    pub(super) notes: Option<String>,
    pub(super) metadata: Value,
    pub(super) updated_at: DateTime<Utc>,
}
