use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct RulePackageRow {
    pub(super) id: Uuid,
    pub(super) format_version: String,
    pub(super) package_key: String,
    pub(super) version: String,
    pub(super) display_name: String,
    pub(super) competition_kind: String,
    pub(super) model_key: String,
    pub(super) model_version: String,
    pub(super) parameter_version: String,
    pub(super) priority: i32,
    pub(super) content_sha256: String,
    pub(super) status: String,
    pub(super) created_at: DateTime<Utc>,
}
