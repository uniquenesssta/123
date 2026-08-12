use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub database_name: String,
    pub server_version: String,
    pub migration_count: i64,
    pub database_size_bytes: i64,
    pub checked_at: DateTime<Utc>,
    pub latency_ms: u128,
}
