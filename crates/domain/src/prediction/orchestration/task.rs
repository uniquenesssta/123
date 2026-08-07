use super::super::P4Horizon;
use super::P4FreezeTaskState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeTaskDraft {
    pub match_id: Uuid,
    pub match_key: String,
    pub horizon: P4Horizon,
    pub kickoff_at: DateTime<Utc>,
    pub data_cutoff_at: DateTime<Utc>,
    pub research_due_at: DateTime<Utc>,
    pub freeze_deadline_at: DateTime<Utc>,
    pub rule_package_id: Uuid,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub competition_profile_id: Uuid,
    pub research_schema_version_id: Uuid,
    pub snapshot_schema_version_id: Uuid,
    pub requested_fact_keys: Vec<String>,
    pub trace_id: Uuid,
    pub state: P4FreezeTaskState,
    pub idempotency_key: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeTaskRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub horizon: P4Horizon,
    pub kickoff_at: DateTime<Utc>,
    pub data_cutoff_at: DateTime<Utc>,
    pub research_due_at: DateTime<Utc>,
    pub freeze_deadline_at: DateTime<Utc>,
    pub rule_package_id: Uuid,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub competition_profile_id: Uuid,
    pub research_schema_version_id: Uuid,
    pub snapshot_schema_version_id: Uuid,
    pub requested_fact_keys: Vec<String>,
    pub trace_id: Uuid,
    pub state: P4FreezeTaskState,
    pub research_run_id: Option<Uuid>,
    pub research_job_id: Option<Uuid>,
    pub freeze_job_id: Option<Uuid>,
    pub snapshot_id: Option<Uuid>,
    pub blockers: Value,
    pub task_fingerprint: String,
    pub idempotency_key: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeTaskEventRecord {
    pub id: Uuid,
    pub task_id: Uuid,
    pub from_state: Option<P4FreezeTaskState>,
    pub to_state: P4FreezeTaskState,
    pub reason: String,
    pub payload: Value,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeTaskTransition {
    pub task_id: Uuid,
    pub expected_state: P4FreezeTaskState,
    pub next_state: P4FreezeTaskState,
    pub reason: String,
    #[serde(default)]
    pub blockers: Value,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub research_run_id: Option<Uuid>,
    #[serde(default)]
    pub research_job_id: Option<Uuid>,
    #[serde(default)]
    pub freeze_job_id: Option<Uuid>,
    #[serde(default)]
    pub snapshot_id: Option<Uuid>,
}
