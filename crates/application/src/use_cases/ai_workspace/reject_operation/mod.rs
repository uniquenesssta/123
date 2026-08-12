use crate::{ports::ai_workspace::ApiWorkspaceOperationPort, ApplicationResult};
use football_domain::ApiWorkspaceOperationRecord;
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    operation_id: Uuid,
    reason: String,
) -> ApplicationResult<ApiWorkspaceOperationRecord>
where
    P: ApiWorkspaceOperationPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let reason = if reason.trim().is_empty() {
        "用户拒绝该数据库提案"
    } else {
        reason.trim()
    };
    Ok(session
        .await?
        .reject_operation(operation_id, reason)
        .await?)
}
