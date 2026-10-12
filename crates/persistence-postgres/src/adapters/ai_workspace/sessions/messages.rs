use super::mapping::message_from_row;
use super::validation::validate_message_draft;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{ApiWorkspaceMessageDraft, ApiWorkspaceMessageRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn append_api_workspace_message(
        &self,
        draft: &ApiWorkspaceMessageDraft,
    ) -> PersistenceResult<ApiWorkspaceMessageRecord> {
        validate_message_draft(draft)?;
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO ai_workspace.messages (
                id, session_id, role, content, structured_payload, citations,
                attachments, provider_response_id, model_id, token_usage
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, session_id, role, content, structured_payload, citations,
                      attachments, provider_response_id, model_id, token_usage, created_at
            "#,
        )
        .bind(id)
        .bind(draft.session_id)
        .bind(draft.role.trim())
        .bind(&draft.content)
        .bind(&draft.structured_payload)
        .bind(&draft.citations)
        .bind(&draft.attachments)
        .bind(draft.provider_response_id.as_deref())
        .bind(draft.model_id.as_deref())
        .bind(&draft.token_usage)
        .fetch_one(&self.pool)
        .await?;
        sqlx::query("UPDATE ai_workspace.sessions SET updated_at = now() WHERE id = $1")
            .bind(draft.session_id)
            .execute(&self.pool)
            .await?;
        message_from_row(&row)
    }
}
