use super::AiWorkspaceService;
use crate::{
    ports::{
        ai_workspace::ApiWorkspaceOperationPort,
        player::{PlayerCatalogPort, PlayerSignalPort},
        team::TeamCatalogPort,
    },
    use_cases::ai_workspace,
    ApplicationResult,
};
use football_domain::{ApiWorkspaceApplyResult, ApiWorkspaceOperationRecord};
use std::future::Future;
use uuid::Uuid;

impl AiWorkspaceService {
    pub(crate) async fn read_operation<P, F>(
        &self,
        session: F,
        operation_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceOperationRecord>
    where
        P: ApiWorkspaceOperationPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::read_operation::execute(session, operation_id).await
    }

    pub(crate) async fn apply_operation<P, F>(
        &self,
        session: F,
        operation_id: Uuid,
    ) -> ApplicationResult<ApiWorkspaceApplyResult>
    where
        P: ApiWorkspaceOperationPort + PlayerCatalogPort + PlayerSignalPort + TeamCatalogPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::apply_operation::execute(session, operation_id).await
    }

    pub(crate) async fn reject_operation<P, F>(
        &self,
        session: F,
        operation_id: Uuid,
        reason: String,
    ) -> ApplicationResult<ApiWorkspaceOperationRecord>
    where
        P: ApiWorkspaceOperationPort,
        F: Future<Output = ApplicationResult<P>>,
    {
        ai_workspace::reject_operation::execute(session, operation_id, reason).await
    }
}
