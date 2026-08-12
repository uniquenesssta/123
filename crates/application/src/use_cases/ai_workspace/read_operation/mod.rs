use crate::{ports::ai_workspace::ApiWorkspaceOperationPort, ApplicationResult};
use football_domain::ApiWorkspaceOperationRecord;
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    operation_id: Uuid,
) -> ApplicationResult<ApiWorkspaceOperationRecord>
where
    P: ApiWorkspaceOperationPort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session.await?.read_operation(operation_id).await?)
}
