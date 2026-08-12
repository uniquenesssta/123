use crate::{ports::release::ReleaseAcceptancePort, ApplicationResult};
use football_domain::ReleaseAcceptanceRun;
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    run_id: Uuid,
) -> ApplicationResult<ReleaseAcceptanceRun>
where
    P: ReleaseAcceptancePort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session.await?.read_run(run_id).await?)
}
