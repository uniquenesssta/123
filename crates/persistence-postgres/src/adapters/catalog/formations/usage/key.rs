use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

pub(crate) struct FormationUsageGroupKey<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) team_id: Option<Uuid>,
    pub(crate) coach_id: Option<Uuid>,
    pub(crate) competition_id: Option<Uuid>,
    pub(crate) window_start: NaiveDate,
    pub(crate) window_end: NaiveDate,
    pub(crate) observed_at: DateTime<Utc>,
}
