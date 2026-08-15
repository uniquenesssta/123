use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct StageContextRow {
    pub(super) competition_id: Uuid,
    pub(super) season_id: Uuid,
    pub(super) stage_id: Uuid,
    pub(super) stage_kind: String,
}

#[derive(Debug, FromRow)]
pub(super) struct SeasonContextRow {
    pub(super) competition_id: Uuid,
    pub(super) season_id: Uuid,
    pub(super) competition_kind: String,
}
