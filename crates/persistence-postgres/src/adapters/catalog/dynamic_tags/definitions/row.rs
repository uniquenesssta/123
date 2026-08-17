use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub(super) struct DynamicTagDefinitionRow {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub default_value: f64,
    pub default_ttl_hours: i32,
    pub is_multiplier: bool,
    pub description: Option<String>,
}
