use crate::EntityReferenceCount;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamForceDeleteRequest {
    pub team_id: Uuid,
    pub confirmation_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamForceDeletePreview {
    pub team_id: Uuid,
    pub label: String,
    pub confirmation_text: String,
    pub total_rows: u64,
    pub references: Vec<EntityReferenceCount>,
    pub warning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamForceDeleteResult {
    pub team_id: Uuid,
    pub label: String,
    pub deleted_match_ids: Vec<Uuid>,
    pub deleted_player_ids: Vec<Uuid>,
    pub deleted_coach_ids: Vec<Uuid>,
    pub deleted_import_batch_ids: Vec<Uuid>,
    pub deleted_counts: std::collections::BTreeMap<String, u64>,
}
