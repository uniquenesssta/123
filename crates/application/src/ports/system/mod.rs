use crate::ports::PortResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

pub trait ClockPort: Send + Sync {
    fn now_utc(&self) -> DateTime<Utc>;
}

#[async_trait]
pub trait FileStoragePort: Send + Sync {
    async fn read(&self, logical_path: &str) -> PortResult<Vec<u8>>;
    async fn write_atomic(&self, logical_path: &str, bytes: &[u8]) -> PortResult<()>;
    async fn exists(&self, logical_path: &str) -> PortResult<bool>;
}
