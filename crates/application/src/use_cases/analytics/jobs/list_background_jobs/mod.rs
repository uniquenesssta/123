use crate::{ports::analytics::JobQueuePort, ApplicationResult};
use football_domain::BackgroundJob;

pub(crate) async fn execute<P>(port: &P, limit: u32) -> ApplicationResult<Vec<BackgroundJob>>
where
    P: JobQueuePort + ?Sized,
{
    Ok(port.list_jobs(limit).await?)
}
