use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundJob {
    pub id: Uuid,
    pub job_type: String,
    pub status: JobStatus,
    pub progress: f64,
    pub payload: Value,
    pub result: Option<Value>,
    pub error_message: Option<String>,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub cancellation_requested: bool,
    pub available_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueJobDraft {
    pub job_type: String,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub available_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: i32,
}

fn default_max_attempts() -> i32 {
    3
}
