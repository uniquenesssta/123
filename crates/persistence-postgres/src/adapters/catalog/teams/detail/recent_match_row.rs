use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamRecentMatchRow {
    pub(super) match_id: Uuid,
    pub(super) opponent_team_id: Uuid,
    pub(super) opponent_team_name: String,
    pub(super) kickoff_time: DateTime<Utc>,
    pub(super) venue_side: String,
    pub(super) status: String,
    pub(super) goals_for: Option<i16>,
    pub(super) goals_against: Option<i16>,
}
