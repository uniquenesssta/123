use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkArchiveFailedItem {
    pub id: Uuid,
    pub label: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkArchiveResult {
    pub entity_type: String,
    pub requested_count: u64,
    pub archived_ids: Vec<Uuid>,
    pub already_archived_ids: Vec<Uuid>,
    pub failed: Vec<BulkArchiveFailedItem>,
}
