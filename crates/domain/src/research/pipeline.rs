use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactPipelineContext {
    pub research_run_id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub horizon: String,
    pub data_cutoff_at: DateTime<Utc>,
    pub trace_id: Uuid,
    pub prompt_version_id: Option<Uuid>,
    pub prompt_version: Option<String>,
    pub schema_version_id: Uuid,
    pub schema_version: String,
    pub home_team_id: Option<Uuid>,
    pub home_team_name: Option<String>,
    pub away_team_id: Option<Uuid>,
    pub away_team_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub competition_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactPipelineSummary {
    pub fact_count: u32,
    pub missing_field_count: u32,
    pub evidence_claim_count: u32,
    pub resolved_entity_count: u32,
    pub ambiguous_entity_count: u32,
    pub unmatched_entity_count: u32,
    pub time_rejected_count: u32,
    pub conflict_count: u32,
    pub auto_resolved_conflict_count: u32,
    pub manual_conflict_count: u32,
    pub routed_count: u32,
    pub blocked_count: u32,
}

impl FactPipelineSummary {
    pub const fn has_blockers(&self) -> bool {
        self.ambiguous_entity_count > 0
            || self.unmatched_entity_count > 0
            || self.time_rejected_count > 0
            || self.manual_conflict_count > 0
            || self.blocked_count > 0
    }
}
