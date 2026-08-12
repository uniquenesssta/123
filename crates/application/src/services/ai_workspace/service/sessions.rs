use super::AiWorkspaceService;
use crate::{
    ports::{ai_workspace::ApiWorkspaceSessionPort, research::ResearchGatewayAuditPort},
    use_cases::ai_workspace,
    ApplicationResult,
};
use football_domain::{
    ApiWorkspaceAttachment, ApiWorkspaceGeneratedFileContent, ApiWorkspaceGeneratedFileDraft,
    ApiWorkspaceMessageDraft, ApiWorkspaceMessageRecord, ApiWorkspaceOperationDraft,
    ApiWorkspaceSessionDetail, ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord,
    OpenAiUsageTotals,
};
use std::future::Future;
use uuid::Uuid;

impl AiWorkspaceService {
    pub(crate) async fn usage_totals<P, F>(&self, session: F) -> ApplicationResult<OpenAiUsageTotals>
    where
        P: ApiWorkspaceSessionPort + ResearchGatewayAuditPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::usage_totals::execute(session).await
    }

    pub(crate) async fn list_sessions<P, F>(
        &self,
        session: F,
        limit: u32,
    ) -> ApplicationResult<Vec<ApiWorkspaceSessionRecord>>
    where
        P: ApiWorkspaceSessionPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::list_sessions::execute(session, limit).await
    }

    pub(crate) async fn create_session<P, F>(
        &self,
        session: F,
        draft: ApiWorkspaceSessionDraft,
    ) -> ApplicationResult<ApiWorkspaceSessionRecord>
    where
        P: ApiWorkspaceSessionPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::create_session::execute(session, draft).await
    }

    pub(crate) async fn archive_session<P, F>(
        &self,
        session: F,
        session_id: Uuid,
    ) -> ApplicationResult<()>
    where
        P: ApiWorkspaceSessionPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::archive_session::execute(session, session_id).await
    }

    pub(crate) async fn read_session<P, F>(
        &self,
        session: F,
        session_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceSessionDetail>
    where
        P: ApiWorkspaceSessionPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::read_session::execute(session, session_id).await
    }

    pub(crate) async fn append_user_message<P, F>(
        &self,
        session: F,
        session_id: Uuid,
        content: String,
        attachments: &[ApiWorkspaceAttachment],
    ) -> ApplicationResult<ApiWorkspaceMessageRecord>
    where
        P: ApiWorkspaceSessionPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::append_user_message::execute(session, session_id, content, attachments).await
    }

    pub(crate) async fn append_assistant_bundle<P, F>(
        &self,
        session: F,
        message: ApiWorkspaceMessageDraft,
        operations: Vec<ApiWorkspaceOperationDraft>,
        files: Vec<ApiWorkspaceGeneratedFileDraft>,
    ) -> ApplicationResult<ApiWorkspaceSessionDetail>
    where
        P: ApiWorkspaceSessionPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::append_assistant_bundle::execute(session, message, operations, files).await
    }

    pub(crate) async fn read_generated_file<P, F>(
        &self,
        session: F,
        file_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceGeneratedFileContent>
    where
        P: ApiWorkspaceSessionPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::read_generated_file::execute(session, file_id).await
    }
}
