use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
use crate::ports::{
    analytics::{AnalyticsJobProgressPayload, AnalyticsJobResult, JobQueuePort},
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use football_domain::{BackgroundJob, EnqueueJobDraft};
use uuid::Uuid;

fn map_serialization_error(error: serde_json::Error) -> PortError {
    PortError::new(PortErrorKind::Serialization, error.to_string())
}

#[async_trait]
impl JobQueuePort for PersistenceStore {
    async fn enqueue(&self, draft: &EnqueueJobDraft) -> PortResult<BackgroundJob> {
        self.enqueue_job(draft).await.map_err(map_persistence_error)
    }

    async fn list_jobs(&self, limit: u32) -> PortResult<Vec<BackgroundJob>> {
        self.list_jobs(limit).await.map_err(map_persistence_error)
    }

    async fn request_cancellation(&self, job_id: Uuid) -> PortResult<BackgroundJob> {
        self.request_job_cancellation(job_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn retry(&self, job_id: Uuid) -> PortResult<BackgroundJob> {
        self.retry_job(job_id).await.map_err(map_persistence_error)
    }

    async fn claim_next_by_types(&self, job_types: &[&str]) -> PortResult<Option<BackgroundJob>> {
        self.claim_next_job_by_types(job_types)
            .await
            .map_err(map_persistence_error)
    }

    async fn update_progress(
        &self,
        job_id: Uuid,
        progress: f64,
        message: &str,
        payload: AnalyticsJobProgressPayload,
    ) -> PortResult<bool> {
        self.update_job_progress(job_id, progress, message, payload.into_value())
            .await
            .map_err(map_persistence_error)
    }

    async fn complete(&self, job_id: Uuid, result: AnalyticsJobResult) -> PortResult<()> {
        let result = result.into_value().map_err(map_serialization_error)?;
        self.complete_job(job_id, result)
            .await
            .map_err(map_persistence_error)
    }

    async fn fail(&self, job_id: Uuid, error_message: &str) -> PortResult<()> {
        self.fail_job(job_id, error_message)
            .await
            .map_err(map_persistence_error)
    }
}
