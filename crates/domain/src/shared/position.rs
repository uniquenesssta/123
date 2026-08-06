use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionReference {
    pub code: String,
    pub name: String,
    pub position_group: String,
    pub sort_order: i16,
}
