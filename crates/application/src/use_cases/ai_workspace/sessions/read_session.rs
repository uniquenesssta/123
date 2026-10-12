use crate::{ports::ai_workspace::ApiWorkspaceSessionPort, ApplicationResult};
use football_domain::ApiWorkspaceSessionDetail;
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    session_id: Uuid,
) -> ApplicationResult<ApiWorkspaceSessionDetail>
where
    P: ApiWorkspaceSessionPort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session.await?.read_session(session_id).await?)
}
