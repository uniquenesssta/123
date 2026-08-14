use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct RoundRow {
    pub(super) id: Uuid,
    pub(super) stage_id: Uuid,
    pub(super) stage_name: String,
    pub(super) code: String,
    pub(super) name: String,
    pub(super) sequence_no: i32,
    pub(super) starts_at: Option<DateTime<Utc>>,
    pub(super) ends_at: Option<DateTime<Utc>>,
}
