use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub(super) struct AbilityDimensionRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub description: Option<String>,
}
