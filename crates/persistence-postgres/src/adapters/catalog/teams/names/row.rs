use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct TeamNameRow {
    pub(super) id: Uuid,
    pub(super) team_id: Uuid,
    pub(super) name: String,
    pub(super) normalized_name: String,
    pub(super) language_code: Option<String>,
    pub(super) valid_from: Option<NaiveDate>,
    pub(super) valid_to: Option<NaiveDate>,
}
