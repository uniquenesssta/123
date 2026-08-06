use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityDimensionRecord {
    pub code: String,
    pub name: String,
    pub category: String,
    pub minimum_value: f64,
    pub maximum_value: f64,
    pub description: Option<String>,
}
