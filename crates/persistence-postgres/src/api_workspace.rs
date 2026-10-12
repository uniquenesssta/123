use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{ApiWorkspaceOperationDraft, ApiWorkspaceOperationRecord};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn recover_interrupted_api_workspace_operations(&self) -> PersistenceResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE ai_workspace.operation_proposals
            SET status = 'manual_review',
                error_message = COALESCE(
                    error_message,
                    '应用进程在正式数据写入结果登记前中断；为避免重复写入，必须人工核对后重新生成提案'
                ),
                decided_at = now()
            WHERE status = 'applying'
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn claim_api_workspace_operation(
        &self,
        operation_id: Uuid,
    ) -> PersistenceResult<ApiWorkspaceOperationRecord> {
        let row = sqlx::query(
            r#"
            UPDATE ai_workspace.operation_proposals
            SET status = 'applying', error_message = NULL, decided_at = NULL
            WHERE id = $1 AND status = 'pending'
            RETURNING id, session_id, message_id, proposal_key, operation_type, payload,
                      rationale, confidence, status, result, error_message,
                      idempotency_key, created_at, decided_at
            "#,
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await?;
        if let Some(row) = row {
            return operation_from_row(&row);
        }
        let existing = self.read_api_workspace_operation(operation_id).await?;
        Err(PersistenceError::InvalidState(format!(
            "数据库提案当前状态为{}，不能再次应用",
            existing.status
        )))
    }

    pub async fn complete_api_workspace_operation(
        &self,
        operation_id: Uuid,
        status: &str,
        result: Value,
        error_message: Option<&str>,
    ) -> PersistenceResult<ApiWorkspaceOperationRecord> {
        if !matches!(status, "applied" | "failed" | "manual_review") {
            return Err(PersistenceError::InvalidState(
                "API数据库提案完成状态无效".to_string(),
            ));
        }
        let row = sqlx::query(
            r#"
            UPDATE ai_workspace.operation_proposals
            SET status = $2, result = $3, error_message = $4, decided_at = now()
            WHERE id = $1 AND status = 'applying'
            RETURNING id, session_id, message_id, proposal_key, operation_type, payload,
                      rationale, confidence, status, result, error_message,
                      idempotency_key, created_at, decided_at
            "#,
        )
        .bind(operation_id)
        .bind(status)
        .bind(result)
        .bind(error_message)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState("API数据库提案不处于应用中状态".to_string())
        })?;
        operation_from_row(&row)
    }

    pub async fn reject_api_workspace_operation(
        &self,
        operation_id: Uuid,
        reason: &str,
    ) -> PersistenceResult<ApiWorkspaceOperationRecord> {
        let row = sqlx::query(
            r#"
            UPDATE ai_workspace.operation_proposals
            SET status = 'rejected', error_message = $2, decided_at = now()
            WHERE id = $1 AND status = 'pending'
            RETURNING id, session_id, message_id, proposal_key, operation_type, payload,
                      rationale, confidence, status, result, error_message,
                      idempotency_key, created_at, decided_at
            "#,
        )
        .bind(operation_id)
        .bind(reason.trim())
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState("只有待确认的数据库提案可以拒绝".to_string())
        })?;
        operation_from_row(&row)
    }

    pub async fn read_api_workspace_operation(
        &self,
        operation_id: Uuid,
    ) -> PersistenceResult<ApiWorkspaceOperationRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, session_id, message_id, proposal_key, operation_type, payload,
                   rationale, confidence, status, result, error_message,
                   idempotency_key, created_at, decided_at
            FROM ai_workspace.operation_proposals
            WHERE id = $1
            "#,
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("API数据库提案不存在".to_string()))?;
        operation_from_row(&row)
    }
}

pub(crate) fn validate_operation_draft(
    draft: &ApiWorkspaceOperationDraft,
) -> PersistenceResult<()> {
    if !matches!(
        draft.operation_type.as_str(),
        "add_player_name"
            | "assign_player_position"
            | "add_player_availability"
            | "add_player_dynamic_tag"
            | "add_player_ability_observation"
            | "add_team_name"
            | "update_team_profile"
    ) || !draft.payload.is_object()
        || draft.proposal_key.trim().is_empty()
        || draft.idempotency_key.trim().is_empty()
        || !(0.0..=1.0).contains(&draft.confidence)
    {
        return Err(PersistenceError::InvalidState(
            "API数据库提案结构无效".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn operation_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ApiWorkspaceOperationRecord> {
    Ok(ApiWorkspaceOperationRecord {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        message_id: row.try_get("message_id")?,
        proposal_key: row.try_get("proposal_key")?,
        operation_type: row.try_get("operation_type")?,
        payload: row.try_get("payload")?,
        rationale: row.try_get("rationale")?,
        confidence: row.try_get("confidence")?,
        status: row.try_get("status")?,
        result: row.try_get("result")?,
        error_message: row.try_get("error_message")?,
        idempotency_key: row.try_get("idempotency_key")?,
        created_at: row.try_get("created_at")?,
        decided_at: row.try_get("decided_at")?,
    })
}
