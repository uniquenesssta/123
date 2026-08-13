use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
use crate::ports::{
    ai_workspace::{
        ApiWorkspaceOperationPort, ApiWorkspaceSessionPort, SerializedApiWorkspaceOperationResult,
    },
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use football_domain::{
    ApiWorkspaceGeneratedFileContent, ApiWorkspaceGeneratedFileDraft, ApiWorkspaceMessageDraft,
    ApiWorkspaceMessageRecord, ApiWorkspaceOperationDraft, ApiWorkspaceOperationRecord,
    ApiWorkspaceSessionDetail, ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord,
    OpenAiUsageTotals,
};
use uuid::Uuid;

#[async_trait]
impl ApiWorkspaceSessionPort for PersistenceStore {
    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals> {
        self.api_workspace_usage_totals()
            .await
            .map_err(map_persistence_error)
    }

    async fn create_session(
        &self,
        draft: &ApiWorkspaceSessionDraft,
    ) -> PortResult<ApiWorkspaceSessionRecord> {
        self.create_api_workspace_session(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_sessions(&self, limit: u32) -> PortResult<Vec<ApiWorkspaceSessionRecord>> {
        self.list_api_workspace_sessions(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_session(&self, session_id: Uuid) -> PortResult<ApiWorkspaceSessionDetail> {
        self.read_api_workspace_session(session_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn archive_session(&self, session_id: Uuid) -> PortResult<()> {
        self.archive_api_workspace_session(session_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_message(
        &self,
        draft: &ApiWorkspaceMessageDraft,
    ) -> PortResult<ApiWorkspaceMessageRecord> {
        self.append_api_workspace_message(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_assistant_bundle(
        &self,
        message: &ApiWorkspaceMessageDraft,
        operations: &[ApiWorkspaceOperationDraft],
        files: &[ApiWorkspaceGeneratedFileDraft],
    ) -> PortResult<ApiWorkspaceSessionDetail> {
        self.append_api_workspace_assistant_bundle(message, operations, files)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_generated_file(
        &self,
        file_id: Uuid,
    ) -> PortResult<ApiWorkspaceGeneratedFileContent> {
        self.read_api_workspace_generated_file(file_id)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ApiWorkspaceOperationPort for PersistenceStore {
    async fn claim_operation(&self, operation_id: Uuid) -> PortResult<ApiWorkspaceOperationRecord> {
        self.claim_api_workspace_operation(operation_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn complete_operation(
        &self,
        operation_id: Uuid,
        status: &str,
        result: &SerializedApiWorkspaceOperationResult,
        error_message: Option<&str>,
    ) -> PortResult<ApiWorkspaceOperationRecord> {
        let result = serde_json::from_str(&result.0).map_err(|error| {
            PortError::new(
                PortErrorKind::Serialization,
                format!("API workspace operation result is not valid JSON: {error}"),
            )
        })?;
        self.complete_api_workspace_operation(operation_id, status, result, error_message)
            .await
            .map_err(map_persistence_error)
    }

    async fn reject_operation(
        &self,
        operation_id: Uuid,
        reason: &str,
    ) -> PortResult<ApiWorkspaceOperationRecord> {
        self.reject_api_workspace_operation(operation_id, reason)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_operation(&self, operation_id: Uuid) -> PortResult<ApiWorkspaceOperationRecord> {
        self.read_api_workspace_operation(operation_id)
            .await
            .map_err(map_persistence_error)
    }
}
