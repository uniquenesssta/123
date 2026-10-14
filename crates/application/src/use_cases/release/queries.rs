use crate::{ports::release::ReleaseAcceptancePort, ApplicationResult};
use football_domain::{ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary};
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn list_release_acceptance_runs<P, F>(
    session: F,
    limit: u32,
) -> ApplicationResult<Vec<ReleaseAcceptanceRunSummary>>
where
    P: ReleaseAcceptancePort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session.await?.list_runs(limit).await?)
}

pub(crate) async fn read_release_acceptance_run<P, F>(
    session: F,
    run_id: Uuid,
) -> ApplicationResult<ReleaseAcceptanceRun>
where
    P: ReleaseAcceptancePort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session.await?.read_run(run_id).await?)
}
