use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct PlayerAvailabilityRow {
    pub id: Uuid,
    pub player_id: Uuid,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub status: String,
    pub reason: Option<String>,
    pub confidence: f64,
    pub valid_from: DateTime<Utc>,
    pub valid_to: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
