use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(in crate::adapters::competition) struct CompetitionRow {
    pub(super) id: Uuid,
    pub(super) code: String,
    pub(super) name: String,
    pub(super) country_code: Option<String>,
    pub(super) timezone: String,
    pub(super) competition_kind: String,
    pub(super) is_active: bool,
    pub(super) metadata: Value,
    pub(super) created_at: DateTime<Utc>,
}
