use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    ApiWorkspaceAssistantFile, ApiWorkspaceGeneratedFileContent, ApiWorkspaceMessageDraft,
    ApiWorkspaceMessageRecord, ApiWorkspaceOperationDraft, ApiWorkspaceOperationRecord,
    ApiWorkspaceSessionDetail, ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord,
};
use uuid::Uuid;

#[async_trait]
pub trait ApiWorkspaceSessionPort: Send + Sync {
    async fn create_session(
        &self,
        draft: &ApiWorkspaceSessionDraft,
    ) -> PortResult<ApiWorkspaceSessionRecord>;
    async fn list_sessions(&self, limit: i64) -> PortResult<Vec<ApiWorkspaceSessionRecord>>;
    async fn read_session(&self, session_id: Uuid) -> PortResult<ApiWorkspaceSessionDetail>;
    async fn archive_session(&self, session_id: Uuid) -> PortResult<ApiWorkspaceSessionRecord>;
    async fn append_message(
        &self,
        draft: &ApiWorkspaceMessageDraft,
    ) -> PortResult<ApiWorkspaceMessageRecord>;
    async fn read_generated_file(
        &self,
        file_id: Uuid,
    ) -> PortResult<ApiWorkspaceGeneratedFileContent>;
    async fn append_assistant_files(
        &self,
        session_id: Uuid,
        files: &[ApiWorkspaceAssistantFile],
    ) -> PortResult<()>;
}

#[async_trait]
pub trait ApiWorkspaceOperationPort: Send + Sync {
    async fn recover_interrupted(&self) -> PortResult<u64>;
    async fn create_operation(
        &self,
        draft: &ApiWorkspaceOperationDraft,
    ) -> PortResult<ApiWorkspaceOperationRecord>;
    async fn claim_operation(&self, operation_id: Uuid) -> PortResult<ApiWorkspaceOperationRecord>;
    async fn complete_operation(
        &self,
        operation_id: Uuid,
    ) -> PortResult<ApiWorkspaceOperationRecord>;
    async fn reject_operation(
        &self,
        operation_id: Uuid,
        reason: &str,
    ) -> PortResult<ApiWorkspaceOperationRecord>;
    async fn read_operation(&self, operation_id: Uuid) -> PortResult<ApiWorkspaceOperationRecord>;
}
