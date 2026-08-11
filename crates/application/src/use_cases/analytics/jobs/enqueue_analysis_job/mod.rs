use super::worker;
use crate::{
    ports::analytics::{AnalyticsPort, JobQueuePort},
    ApplicationResult,
};
use football_domain::{BackgroundJob, EnqueueJobDraft};

pub(crate) async fn execute<P>(port: &P, draft: EnqueueJobDraft) -> ApplicationResult<BackgroundJob>
where
    P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
{
    let job = port.enqueue(&draft).await?;
    worker::spawn(port.clone());
    Ok(job)
}
