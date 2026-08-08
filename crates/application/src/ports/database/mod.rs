use crate::ports::PortResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseHealthSnapshot {
    pub connected: bool,
    pub database_name: String,
    pub server_version: String,
    pub migration_count: i64,
    pub database_size_bytes: i64,
    pub checked_at: DateTime<Utc>,
    pub latency_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseStatistics {
    pub competitions: i64,
    pub teams: i64,
    pub players: i64,
    pub matches: i64,
    pub model_runs: i64,
    pub rule_packages: i64,
    pub route_bindings: i64,
    pub ability_observations: i64,
    pub pending_ability_updates: i64,
    pub data_providers: i64,
    pub availability_records: i64,
    pub active_lineups: i64,
    pub large_counts_are_estimates: bool,
}

#[async_trait]
pub trait DatabaseLifecyclePort: Send + Sync {
    async fn migrate(&self) -> PortResult<()>;
    async fn recover_interrupted_work(&self) -> PortResult<()>;
    async fn close(&self) -> PortResult<()>;
}

#[async_trait]
pub trait DatabaseObservabilityPort: Send + Sync {
    async fn health(&self) -> PortResult<DatabaseHealthSnapshot>;
    async fn statistics(&self) -> PortResult<DatabaseStatistics>;
}
