use super::coordinator::run_queue;
use crate::ports::analytics::{AnalyticsPort, JobQueuePort};

pub(crate) fn spawn<P>(port: P)
where
    P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
{
    tokio::spawn(async move {
        run_queue(&port).await;
    });
}
