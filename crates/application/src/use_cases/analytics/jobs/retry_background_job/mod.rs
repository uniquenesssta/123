use super::worker;
use crate::{
    ports::analytics::{AnalyticsPort, JobQueuePort},
    ApplicationResult,
};
use football_domain::BackgroundJob;
use uuid::Uuid;

pub(crate) async fn execute<P>(port: &P, job_id: Uuid) -> ApplicationResult<BackgroundJob>
where
    P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
{
    let job = port.retry(job_id).await?;
    worker::spawn(port.clone());
    Ok(job)
}
