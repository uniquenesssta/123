use super::initialize;
use crate::composition::{database_health_from_snapshot, DatabaseHealth, DatabaseOptions};
use crate::{ApplicationError, ApplicationResult, ApplicationService};
use std::sync::Arc;

pub(crate) async fn execute(
    application: &Arc<ApplicationService>,
    options: DatabaseOptions,
) -> ApplicationResult<DatabaseHealth> {
    let prepared = application.database.prepare_connection(&options).await?;
    if let Err(error) = initialize::execute(application, prepared.session()).await {
        prepared.close().await;
        return Err(error);
    }

    let health = match prepared.health().await {
        Ok(health) => database_health_from_snapshot(health),
        Err(error) => {
            prepared.close().await;
            return Err(error.into());
        }
    };
    application.database.activate(prepared).await?;

    let session = application
        .database
        .active_session()
        .await
        .ok_or(ApplicationError::DatabaseNotConnected)?;
    application.analytics.start_job_worker(session);
    application.p4_orchestration.start(Arc::clone(application));
    Ok(health)
}
