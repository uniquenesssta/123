use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(in crate::adapters::competition::bindings) struct BindingRow {
    pub id: Uuid,
    pub binding_name: String,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub competition_kind: Option<String>,
    pub rule_package_id: Uuid,
    pub rule_package_name: String,
    pub model_key: String,
    pub priority: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
