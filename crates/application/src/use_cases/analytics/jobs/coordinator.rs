use super::execution::execute_job;
use crate::ports::analytics::{AnalyticsPort, JobQueuePort};

pub(super) async fn run_queue<P>(port: &P)
where
    P: AnalyticsPort + JobQueuePort + ?Sized,
{
    loop {
        let job = match port
            .claim_next_by_types(&[
                "refresh_analytics",
                "data_quality_scan",
                "query_performance_scan",
                "full_analysis_refresh",
            ])
            .await
        {
            Ok(Some(job)) => job,
            Ok(None) => break,
            Err(_) => break,
        };
        let result = execute_job(port, &job.job_type, &job.payload, job.id).await;
        match result {
            Ok(value) => {
                let _ = port.complete(job.id, value).await;
            }
            Err(error) => {
                let _ = port.fail(job.id, &error.to_string()).await;
            }
        }
    }
}
