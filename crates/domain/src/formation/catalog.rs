use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationRecord {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub line_structure: String,
    pub slot_definition: Value,
    pub is_builtin: bool,
    pub is_active: bool,
    pub sort_order: i16,
    pub metadata: Value,
}
