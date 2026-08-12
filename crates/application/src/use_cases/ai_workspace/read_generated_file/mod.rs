use crate::{ports::ai_workspace::ApiWorkspaceSessionPort, ApplicationResult};
use football_domain::ApiWorkspaceGeneratedFileContent;
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    file_id: Uuid,
) -> ApplicationResult<ApiWorkspaceGeneratedFileContent>
where
    P: ApiWorkspaceSessionPort,
    F: Future<Output = ApplicationResult<P>>,
{
    Ok(session.await?.read_generated_file(file_id).await?)
}
