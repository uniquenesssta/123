use chrono::{DateTime, NaiveDate, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct PlayerRecordRow {
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub date_of_birth: Option<NaiveDate>,
    pub nationality_code: Option<String>,
    pub preferred_foot: String,
    pub height_cm: Option<i16>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
