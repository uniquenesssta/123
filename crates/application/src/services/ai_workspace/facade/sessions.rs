use crate::{ApplicationResult, ApplicationService};
use football_domain::{
    ApiWorkspaceAttachment, ApiWorkspaceGeneratedFileContent, ApiWorkspaceGeneratedFileDraft,
    ApiWorkspaceMessageDraft, ApiWorkspaceMessageRecord, ApiWorkspaceOperationDraft,
    ApiWorkspaceSessionDetail, ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord,
    OpenAiUsageTotals,
};
use uuid::Uuid;

impl ApplicationService {
    pub async fn api_workspace_openai_usage_totals(&self) -> ApplicationResult<OpenAiUsageTotals> {
        self.ai_workspace
            .usage_totals(self.ai_workspace_session())
            .await
    }

    pub async fn list_api_workspace_sessions(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<ApiWorkspaceSessionRecord>> {
        self.ai_workspace
            .list_sessions(self.ai_workspace_session(), limit)
            .await
    }

    pub async fn create_api_workspace_session(
        &self,
        draft: ApiWorkspaceSessionDraft,
    ) -> ApplicationResult<ApiWorkspaceSessionRecord> {
        self.ai_workspace
            .create_session(self.ai_workspace_session(), draft)
            .await
    }

    pub async fn archive_api_workspace_session(&self, session_id: Uuid) -> ApplicationResult<()> {
        self.ai_workspace
            .archive_session(self.ai_workspace_session(), session_id)
            .await
    }

    pub async fn read_api_workspace_session(
        &self,
        session_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceSessionDetail> {
        self.ai_workspace
            .read_session(self.ai_workspace_session(), session_id)
            .await
    }

    pub async fn append_api_workspace_user_message(
        &self,
        session_id: Uuid,
        content: String,
        attachments: &[ApiWorkspaceAttachment],
    ) -> ApplicationResult<ApiWorkspaceMessageRecord> {
        self.ai_workspace
            .append_user_message(
                self.ai_workspace_session(),
                session_id,
                content,
                attachments,
            )
            .await
    }

    pub async fn append_api_workspace_assistant_bundle(
        &self,
        message: ApiWorkspaceMessageDraft,
        operations: Vec<ApiWorkspaceOperationDraft>,
        files: Vec<ApiWorkspaceGeneratedFileDraft>,
    ) -> ApplicationResult<ApiWorkspaceSessionDetail> {
        self.ai_workspace
            .append_assistant_bundle(self.ai_workspace_session(), message, operations, files)
            .await
    }

    pub async fn read_api_workspace_generated_file(
        &self,
        file_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceGeneratedFileContent> {
        self.ai_workspace
            .read_generated_file(self.ai_workspace_session(), file_id)
            .await
    }
}
