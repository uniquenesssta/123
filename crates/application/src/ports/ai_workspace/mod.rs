use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    ApiWorkspaceGeneratedFileContent, ApiWorkspaceGeneratedFileDraft, ApiWorkspaceMessageDraft,
    ApiWorkspaceMessageRecord, ApiWorkspaceOperationDraft, ApiWorkspaceOperationRecord,
    ApiWorkspaceSessionDetail, ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord,
    OpenAiUsageTotals,
};
use uuid::Uuid;

#[async_trait]
pub trait ApiWorkspaceSessionPort: Send + Sync {
    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals>;
    async fn create_session(
        &self,
        draft: &ApiWorkspaceSessionDraft,
    ) -> PortResult<ApiWorkspaceSessionRecord>;
    async fn list_sessions(&self, limit: u32) -> PortResult<Vec<ApiWorkspaceSessionRecord>>;
    async fn read_session(&self, session_id: Uuid) -> PortResult<ApiWorkspaceSessionDetail>;
    async fn archive_session(&self, session_id: Uuid) -> PortResult<()>;
    async fn append_message(
        &self,
        draft: &ApiWorkspaceMessageDraft,
    ) -> PortResult<ApiWorkspaceMessageRecord>;
    async fn append_assistant_bundle(
        &self,
        message: &ApiWorkspaceMessageDraft,
        operations: &[ApiWorkspaceOperationDraft],
        files: &[ApiWorkspaceGeneratedFileDraft],
    ) -> PortResult<ApiWorkspaceSessionDetail>;
    async fn read_generated_file(
        &self,
        file_id: Uuid,
    ) -> PortResult<ApiWorkspaceGeneratedFileContent>;
}

#[async_trait]
pub trait ApiWorkspaceOperationPort: Send + Sync {
    async fn claim_operation(&self, operation_id: Uuid) -> PortResult<ApiWorkspaceOperationRecord>;
    async fn complete_operation(
        &self,
        operation_id: Uuid,
        status: &str,
        result: serde_json::Value,
        error_message: Option<&str>,
    ) -> PortResult<ApiWorkspaceOperationRecord>;
    async fn reject_operation(
        &self,
        operation_id: Uuid,
        reason: &str,
    ) -> PortResult<ApiWorkspaceOperationRecord>;
    async fn read_operation(&self, operation_id: Uuid) -> PortResult<ApiWorkspaceOperationRecord>;
}
