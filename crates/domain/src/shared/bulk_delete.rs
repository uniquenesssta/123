use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDeleteBlockedItem {
    pub id: Uuid,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkDeleteResult {
    pub requested_count: u64,
    pub deleted_ids: Vec<Uuid>,
    pub blocked: Vec<BulkDeleteBlockedItem>,
}
