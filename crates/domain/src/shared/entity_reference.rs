use crate::{default_team_page_limit, default_true};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDeletionCheck {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub label: String,
    pub exists: bool,
    pub can_permanently_delete: bool,
    pub must_archive: bool,
    pub references: Vec<EntityReferenceCount>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReferenceCount {
    pub relation: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReferenceQuery {
    pub entity_type: String,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default = "default_true")]
    pub active_only: bool,
    #[serde(default = "default_team_page_limit")]
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityReferenceRecord {
    pub entity_type: String,
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub country_code: Option<String>,
    pub nationality_code: Option<String>,
    pub date_of_birth: Option<chrono::NaiveDate>,
    pub status: String,
    pub aliases: Vec<String>,
    pub external_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntityIdDraft {
    pub provider_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub external_id: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEntityIdRecord {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub external_id: String,
    pub metadata: Value,
}
