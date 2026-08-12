mod dispatch;
mod metadata;
mod payload;

use crate::{
    ports::{
        ai_workspace::ApiWorkspaceOperationPort,
        player::{PlayerCatalogPort, PlayerSignalPort},
        team::TeamCatalogPort,
    },
    ApplicationResult,
};
use football_domain::ApiWorkspaceApplyResult;
use serde_json::json;
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    operation_id: Uuid,
) -> ApplicationResult<ApiWorkspaceApplyResult>
where
    P: ApiWorkspaceOperationPort + PlayerCatalogPort + PlayerSignalPort + TeamCatalogPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let port = session.await?;
    let operation = port.claim_operation(operation_id).await?;
    let apply_result = dispatch::execute(&port, &operation).await;
    match apply_result {
        Ok(result) => {
            let record = port
                .complete_operation(operation_id, "applied", result.clone(), None)
                .await?;
            Ok(ApiWorkspaceApplyResult {
                operation_id,
                operation_type: record.operation_type,
                status: record.status,
                result,
                error_message: None,
            })
        }
        Err(error) => {
            let message = error.to_string();
            let record = port
                .complete_operation(operation_id, "failed", json!({}), Some(&message))
                .await?;
            Ok(ApiWorkspaceApplyResult {
                operation_id,
                operation_type: record.operation_type,
                status: record.status,
                result: json!({}),
                error_message: Some(message),
            })
        }
    }
}
