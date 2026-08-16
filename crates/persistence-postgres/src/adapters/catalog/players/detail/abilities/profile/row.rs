use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct PlayerAbilityProfileRow {
    pub player_id: Uuid,
    pub abilities: Value,
    pub average_value: Option<f64>,
    pub average_confidence: Option<f64>,
    pub dimension_count: i32,
    pub latest_observed_at: Option<DateTime<Utc>>,
    pub next_expiry_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}
