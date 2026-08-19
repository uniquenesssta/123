use chrono::NaiveDate;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct TeamPlayerPeriodRow {
    pub id: Uuid,
    pub team_id: Uuid,
    pub team_name: String,
    pub player_id: Uuid,
    pub player_name: String,
    pub season_id: Option<Uuid>,
    pub season_name: Option<String>,
    pub squad_number: Option<i16>,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub registration_status: String,
}
