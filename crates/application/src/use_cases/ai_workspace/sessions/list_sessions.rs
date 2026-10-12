use crate::{ports::ai_workspace::ApiWorkspaceSessionPort, ApplicationResult};
use football_domain::ApiWorkspaceSessionRecord;
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    limit: u32,
) -> ApplicationResult<Vec<ApiWorkspaceSessionRecord>>
where
    P: ApiWorkspaceSessionPort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session.await?.list_sessions(limit).await?)
}
