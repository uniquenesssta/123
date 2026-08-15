use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamSquadRow {
    pub(super) player_id: Uuid,
    pub(super) player_name: String,
    pub(super) localized_name: Option<String>,
    pub(super) position_code: Option<String>,
    pub(super) role_code: Option<String>,
    pub(super) squad_number: Option<i16>,
    pub(super) registration_status: String,
    pub(super) availability_status: Option<String>,
    pub(super) ability_average: Option<f64>,
}
