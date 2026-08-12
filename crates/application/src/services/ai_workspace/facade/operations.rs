use crate::{ApplicationResult, ApplicationService};
use football_domain::{ApiWorkspaceApplyResult, ApiWorkspaceOperationRecord};
use uuid::Uuid;

impl ApplicationService {
    pub async fn read_api_workspace_operation(
        &self,
        operation_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceOperationRecord> {
        self.ai_workspace
            .read_operation(self.ai_workspace_session(), operation_id)
            .await
    }

    pub async fn apply_api_workspace_operation(
        &self,
        operation_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceApplyResult> {
        self.ai_workspace
            .apply_operation(self.ai_workspace_session(), operation_id)
            .await
    }

    pub async fn reject_api_workspace_operation(
        &self,
        operation_id: Uuid,
        reason: String,
    ) -> ApplicationResult<ApiWorkspaceOperationRecord> {
        self.ai_workspace
            .reject_operation(self.ai_workspace_session(), operation_id, reason)
            .await
    }
}
