use crate::{ports::ai_workspace::ApiWorkspaceSessionPort, ApplicationResult};
use football_domain::{
    ApiWorkspaceGeneratedFileDraft, ApiWorkspaceMessageDraft, ApiWorkspaceOperationDraft,
    ApiWorkspaceSessionDetail,
};
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    message: ApiWorkspaceMessageDraft,
    operations: Vec<ApiWorkspaceOperationDraft>,
    files: Vec<ApiWorkspaceGeneratedFileDraft>,
) -> ApplicationResult<ApiWorkspaceSessionDetail>
where
    P: ApiWorkspaceSessionPort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session
        .await?
        .append_assistant_bundle(&message, &operations, &files)
        .await?)
}
