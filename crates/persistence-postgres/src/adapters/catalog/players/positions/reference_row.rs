use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub(super) struct PositionReferenceRow {
    pub code: String,
    pub name: String,
    pub position_group: String,
    pub sort_order: i16,
}
