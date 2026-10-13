use super::worker;
use crate::{
    ports::analytics::{AnalyticsPort, JobQueuePort},
    ApplicationResult,
};
use football_domain::{BackgroundJob, EnqueueJobDraft};
use uuid::Uuid;

pub(crate) async fn enqueue_analysis_job<P>(
    port: &P,
    draft: EnqueueJobDraft,
) -> ApplicationResult<BackgroundJob>
where
    P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
{
    let job = port.enqueue(&draft).await?;
    worker::spawn(port.clone());
    Ok(job)
}

pub(crate) async fn list_background_jobs<P>(
    port: &P,
    limit: u32,
) -> ApplicationResult<Vec<BackgroundJob>>
where
    P: JobQueuePort + ?Sized,
{
    Ok(port.list_jobs(limit).await?)
}

pub(crate) async fn cancel_background_job<P>(
    port: &P,
    job_id: Uuid,
) -> ApplicationResult<BackgroundJob>
where
    P: JobQueuePort + ?Sized,
{
    Ok(port.request_cancellation(job_id).await?)
}

pub(crate) async fn retry_background_job<P>(
    port: &P,
    job_id: Uuid,
) -> ApplicationResult<BackgroundJob>
where
    P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
{
    let job = port.retry(job_id).await?;
    worker::spawn(port.clone());
    Ok(job)
}
