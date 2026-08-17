use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(in crate::adapters::catalog::dynamic_tags) struct PlayerDynamicTagRow {
    pub id: Uuid,
    pub player_id: Uuid,
    pub tag_code: String,
    pub tag_name: String,
    pub category: String,
    pub value: f64,
    pub label: Option<String>,
    pub confidence: f64,
    pub observed_at: DateTime<Utc>,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub position_code: Option<String>,
    pub opponent_team_id: Option<Uuid>,
    pub opponent_team_name: Option<String>,
    pub sample_size: i32,
    pub source_type: String,
    pub calculation_version: String,
    pub metadata: Value,
}
