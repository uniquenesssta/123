use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(in crate::adapters::catalog::players) struct PlayerPositionRow {
    pub id: Uuid,
    pub player_id: Uuid,
    pub position_code: String,
    pub position_name: String,
    pub position_group: String,
    pub proficiency: f64,
    pub default_role_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
}
