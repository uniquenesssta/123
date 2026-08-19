use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct SeasonTeamMembershipRow {
    pub season_id: Uuid,
    pub team_id: Uuid,
    pub registration_status: String,
}
