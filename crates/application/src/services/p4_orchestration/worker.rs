use crate::{ApplicationError, ApplicationService};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

const P4_WORKER_POLL_SECONDS: u64 = 30;

pub(super) fn spawn(application: Arc<ApplicationService>) {
    if !application.p4_orchestration.try_mark_running() {
        return;
    }
    tokio::spawn(async move {
        loop {
            match application.process_next_p4_orchestration_job().await {
                Ok(Some(_)) => continue,
                Ok(None) => sleep(Duration::from_secs(P4_WORKER_POLL_SECONDS)).await,
                Err(ApplicationError::DatabaseNotConnected) => break,
                Err(_) => sleep(Duration::from_secs(P4_WORKER_POLL_SECONDS)).await,
            }
        }
        application.p4_orchestration.mark_stopped();
    });
}
