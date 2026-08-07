use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformanceFinding {
    pub schema_name: String,
    pub table_name: String,
    pub estimated_rows: i64,
    pub table_size_bytes: i64,
    pub sequential_scans: i64,
    pub index_scans: i64,
    pub dead_rows: i64,
    pub last_analyze: Option<DateTime<Utc>>,
    pub severity: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformanceSummary {
    pub captured_at: Option<DateTime<Utc>>,
    pub database_size_bytes: i64,
    pub tables: Vec<QueryPerformanceFinding>,
}
