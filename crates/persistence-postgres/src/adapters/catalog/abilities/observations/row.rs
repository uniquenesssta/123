use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(in crate::adapters::catalog) struct PlayerAbilityObservationRow {
    pub id: Uuid,
    pub player_id: Uuid,
    pub dimension_code: String,
    pub dimension_name: String,
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
