use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct SeasonRow {
    pub(super) id: Uuid,
    pub(super) competition_id: Uuid,
    pub(super) competition_name: String,
    pub(super) name: String,
    pub(super) starts_on: Option<NaiveDate>,
    pub(super) ends_on: Option<NaiveDate>,
    pub(super) status: String,
}
