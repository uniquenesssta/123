use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeReadiness {
    pub task_id: Uuid,
    pub ready: bool,
    pub research_status: Option<String>,
    pub requested_fact_count: u32,
    pub routed_fact_count: u32,
    pub missing_fact_count: u32,
    pub ignored_fact_count: u32,
    pub blocked_fact_count: u32,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4RoutedFact {
    pub route_key: String,
    pub field_key: String,
    pub target_module: String,
    pub target_slot: String,
    pub route_status: String,
    pub verification_state: String,
    pub selected_evidence_ids: Vec<Uuid>,
    pub selected_value: Value,
    pub reason: String,
}
