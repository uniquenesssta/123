use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{analytics::JobQueuePort, PortResult};
use async_trait::async_trait;
use football_domain::{BackgroundJob, EnqueueJobDraft};
use uuid::Uuid;

#[async_trait]
impl JobQueuePort for ActiveDatabase {
    async fn enqueue(&self, draft: &EnqueueJobDraft) -> PortResult<BackgroundJob> {
        self.transition_store()
            .enqueue_job(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_jobs(&self, limit: i64) -> PortResult<Vec<BackgroundJob>> {
        let limit = u32::try_from(limit.clamp(1, 500)).unwrap_or(500);
        self.transition_store()
            .list_jobs(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn request_cancellation(&self, job_id: Uuid) -> PortResult<BackgroundJob> {
        self.transition_store()
            .request_job_cancellation(job_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn retry(&self, job_id: Uuid) -> PortResult<BackgroundJob> {
        self.transition_store()
            .retry_job(job_id)
            .await
            .map_err(map_persistence_error)
    }
}
