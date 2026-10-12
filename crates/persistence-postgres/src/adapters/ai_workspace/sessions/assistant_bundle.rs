use super::validation::{validate_generated_file_draft, validate_message_draft};
use crate::api_workspace::validate_operation_draft;
use crate::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ApiWorkspaceGeneratedFileDraft, ApiWorkspaceMessageDraft, ApiWorkspaceOperationDraft,
    ApiWorkspaceSessionDetail,
};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn append_api_workspace_assistant_bundle(
        &self,
        message: &ApiWorkspaceMessageDraft,
        operations: &[ApiWorkspaceOperationDraft],
        files: &[ApiWorkspaceGeneratedFileDraft],
    ) -> PersistenceResult<ApiWorkspaceSessionDetail> {
        validate_message_draft(message)?;
        if message.role != "assistant" {
            return Err(PersistenceError::InvalidState(
                "API结构化结果必须以assistant角色写入".to_string(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        let message_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO ai_workspace.messages (
                id, session_id, role, content, structured_payload, citations,
                attachments, provider_response_id, model_id, token_usage
            ) VALUES ($1, $2, 'assistant', $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(message_id)
        .bind(message.session_id)
        .bind(&message.content)
        .bind(&message.structured_payload)
        .bind(&message.citations)
        .bind(&message.attachments)
        .bind(message.provider_response_id.as_deref())
        .bind(message.model_id.as_deref())
        .bind(&message.token_usage)
        .execute(&mut *tx)
        .await?;

        for draft in operations {
            if draft.session_id != message.session_id {
                return Err(PersistenceError::InvalidState(
                    "API数据库提案与回答会话身份不一致".to_string(),
                ));
            }
            validate_operation_draft(draft)?;
            let fingerprint = sha256_json(&json!({
                "session_id": draft.session_id,
                "message_id": message_id,
                "proposal_key": &draft.proposal_key,
                "operation_type": &draft.operation_type,
                "payload": &draft.payload,
                "rationale": &draft.rationale,
                "confidence": draft.confidence,
                "idempotency_key": draft.idempotency_key
            }))?;
            sqlx::query(
                r#"
                INSERT INTO ai_workspace.operation_proposals (
                    id, session_id, message_id, proposal_key, operation_type, payload,
                    rationale, confidence, idempotency_key, operation_fingerprint
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(draft.session_id)
            .bind(message_id)
            .bind(draft.proposal_key.trim())
            .bind(draft.operation_type.trim())
            .bind(&draft.payload)
            .bind(draft.rationale.trim())
            .bind(draft.confidence)
            .bind(draft.idempotency_key.trim())
            .bind(fingerprint)
            .execute(&mut *tx)
            .await?;
        }

        for draft in files {
            if draft.session_id != message.session_id {
                return Err(PersistenceError::InvalidState(
                    "API生成文件与回答会话身份不一致".to_string(),
                ));
            }
            validate_generated_file_draft(draft)?;
            sqlx::query(
                r#"
                INSERT INTO ai_workspace.generated_files (
                    id, session_id, message_id, filename, media_type, content, content_sha256
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(draft.session_id)
            .bind(message_id)
            .bind(draft.filename.trim())
            .bind(draft.media_type.trim())
            .bind(&draft.content)
            .bind(draft.content_sha256.trim())
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query("UPDATE ai_workspace.sessions SET updated_at = now() WHERE id = $1")
            .bind(message.session_id)
            .execute(&mut *tx)
            .await?;
        write_audit_event(
            &mut tx,
            "api_workspace_response_recorded",
            "api_workspace_session",
            Some(message.session_id.to_string()),
            json!({
                "message_id": message_id,
                "operation_count": operations.len(),
                "file_count": files.len(),
                "provider_response_id": message.provider_response_id
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_api_workspace_session(message.session_id).await
    }
}
