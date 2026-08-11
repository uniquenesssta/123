use crate::{ports::analytics::JobQueuePort, ApplicationResult};
use football_domain::BackgroundJob;
use uuid::Uuid;

pub(crate) async fn execute<P>(port: &P, job_id: Uuid) -> ApplicationResult<BackgroundJob>
where
    P: JobQueuePort + ?Sized,
{
    Ok(port.request_cancellation(job_id).await?)
}
