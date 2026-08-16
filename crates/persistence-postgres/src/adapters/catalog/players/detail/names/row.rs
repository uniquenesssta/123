use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct PlayerNameRow {
    pub id: Uuid,
    pub player_id: Uuid,
    pub name: String,
    pub normalized_name: String,
    pub language_code: Option<String>,
    pub is_primary: bool,
    pub valid_from: Option<NaiveDate>,
    pub valid_to: Option<NaiveDate>,
}
