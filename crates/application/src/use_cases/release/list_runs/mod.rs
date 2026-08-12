use crate::{ports::release::ReleaseAcceptancePort, ApplicationResult};
use football_domain::ReleaseAcceptanceRunSummary;
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    limit: u32,
) -> ApplicationResult<Vec<ReleaseAcceptanceRunSummary>>
where
    P: ReleaseAcceptancePort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session.await?.list_runs(limit).await?)
}
