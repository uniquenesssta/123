use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct StageRow {
    pub(super) id: Uuid,
    pub(super) season_id: Uuid,
    pub(super) season_name: String,
    pub(super) competition_id: Uuid,
    pub(super) competition_name: String,
    pub(super) code: String,
    pub(super) name: String,
    pub(super) stage_kind: String,
    pub(super) sequence_no: i32,
}
