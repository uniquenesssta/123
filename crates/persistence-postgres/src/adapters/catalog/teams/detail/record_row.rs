use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(in crate::adapters::catalog::teams) struct TeamRecordRow {
    pub(in crate::adapters::catalog::teams) id: Uuid,
    pub(in crate::adapters::catalog::teams) canonical_name: String,
    pub(in crate::adapters::catalog::teams) normalized_name: String,
    pub(in crate::adapters::catalog::teams) country_code: Option<String>,
    pub(in crate::adapters::catalog::teams) is_active: bool,
    pub(in crate::adapters::catalog::teams) created_at: DateTime<Utc>,
}
