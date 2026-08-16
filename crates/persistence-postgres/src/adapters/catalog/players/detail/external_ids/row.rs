use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct ExternalEntityIdRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub external_id: String,
    pub metadata: Value,
}
