use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterPromotionRequest {
    pub candidate_id: Uuid,
    #[serde(default)]
    pub decided_by: Option<String>,
    pub decision_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRollbackRequest {
    pub candidate_id: Uuid,
    #[serde(default)]
    pub decided_by: Option<String>,
    pub decision_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterPromotionDecisionRecord {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub decision: String,
    pub previous_binding_state: Value,
    pub new_binding_state: Value,
    pub decided_by: Option<String>,
    pub decision_note: String,
    pub created_at: DateTime<Utc>,
}
