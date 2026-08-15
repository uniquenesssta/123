use uuid::Uuid;

#[derive(Debug, sqlx::FromRow)]
pub(super) struct TeamOptionRow {
    pub(super) id: Uuid,
    pub(super) canonical_name: String,
    pub(super) country_code: Option<String>,
    pub(super) team_type: String,
}
