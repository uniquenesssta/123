use super::mapping::{file_from_row, message_from_row, session_from_row};
use crate::api_workspace::operation_from_row;
use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ApiWorkspaceSessionDetail, ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord,
};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_api_workspace_session(
        &self,
        draft: &ApiWorkspaceSessionDraft,
    ) -> PersistenceResult<ApiWorkspaceSessionRecord> {
        if draft.profile_id.trim().is_empty()
            || draft.preset_key.trim().is_empty()
            || draft.title.trim().is_empty()
        {
            return Err(PersistenceError::InvalidState(
                "API协作会话缺少配置、预设或标题".to_string(),
            ));
        }
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO ai_workspace.sessions (
                id, profile_id, preset_key, title, match_id, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(draft.profile_id.trim())
        .bind(draft.preset_key.trim())
        .bind(draft.title.trim())
        .bind(draft.match_id)
        .bind(&draft.metadata)
        .execute(&self.pool)
        .await?;
        self.read_api_workspace_session_summary(id).await
    }

    pub async fn list_api_workspace_sessions(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<ApiWorkspaceSessionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT session.id, session.profile_id, session.preset_key, session.title,
                   session.match_id, session.metadata, session.status, session.created_at, session.updated_at,
                   CASE WHEN fixture.id IS NULL THEN NULL
                        ELSE home.canonical_name || ' vs ' || away.canonical_name
                   END AS match_label,
                   count(DISTINCT message.id)::bigint AS message_count,
                   count(DISTINCT operation.id) FILTER (WHERE operation.status = 'pending')::bigint
                       AS pending_operation_count
            FROM ai_workspace.sessions session
            LEFT JOIN football.matches fixture ON fixture.id = session.match_id
            LEFT JOIN football.teams home ON home.id = fixture.home_team_id
            LEFT JOIN football.teams away ON away.id = fixture.away_team_id
            LEFT JOIN ai_workspace.messages message ON message.session_id = session.id
            LEFT JOIN ai_workspace.operation_proposals operation ON operation.session_id = session.id
            WHERE session.status = 'active'
            GROUP BY session.id, fixture.id, home.canonical_name, away.canonical_name
            ORDER BY session.updated_at DESC, session.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(session_from_row).collect()
    }

    pub async fn archive_api_workspace_session(&self, session_id: Uuid) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            r#"
            UPDATE ai_workspace.sessions
            SET status = 'archived', updated_at = now()
            WHERE id = $1 AND status = 'active'
            "#,
        )
        .bind(session_id)
        .execute(&mut *tx)
        .await?;
        if result.rows_affected() == 0 {
            return Err(PersistenceError::InvalidState(format!(
                "AI问答会话不存在或已归档：{session_id}"
            )));
        }
        write_audit_event(
            &mut tx,
            "api_workspace_session_archived",
            "ai_workspace_session",
            Some(session_id.to_string()),
            json!({"status": "archived"}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn read_api_workspace_session(
        &self,
        session_id: Uuid,
    ) -> PersistenceResult<ApiWorkspaceSessionDetail> {
        let session = self.read_api_workspace_session_summary(session_id).await?;
        let message_rows = sqlx::query(
            r#"
            SELECT id, session_id, role, content, structured_payload, citations,
                   attachments, provider_response_id, model_id, token_usage, created_at
            FROM ai_workspace.messages
            WHERE session_id = $1
            ORDER BY created_at, id
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;
        let operation_rows = sqlx::query(
            r#"
            SELECT id, session_id, message_id, proposal_key, operation_type, payload,
                   rationale, confidence, status, result, error_message,
                   idempotency_key, created_at, decided_at
            FROM ai_workspace.operation_proposals
            WHERE session_id = $1
            ORDER BY created_at, id
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;
        let file_rows = sqlx::query(
            r#"
            SELECT id, session_id, message_id, filename, media_type, content_sha256,
                   octet_length(content)::bigint AS size_bytes, created_at
            FROM ai_workspace.generated_files
            WHERE session_id = $1
            ORDER BY created_at, id
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(ApiWorkspaceSessionDetail {
            session,
            messages: message_rows
                .iter()
                .map(message_from_row)
                .collect::<PersistenceResult<_>>()?,
            operations: operation_rows
                .iter()
                .map(operation_from_row)
                .collect::<PersistenceResult<_>>()?,
            files: file_rows
                .iter()
                .map(file_from_row)
                .collect::<PersistenceResult<_>>()?,
        })
    }

    async fn read_api_workspace_session_summary(
        &self,
        session_id: Uuid,
    ) -> PersistenceResult<ApiWorkspaceSessionRecord> {
        let row = sqlx::query(
            r#"
            SELECT session.id, session.profile_id, session.preset_key, session.title,
                   session.match_id, session.metadata, session.status, session.created_at, session.updated_at,
                   CASE WHEN fixture.id IS NULL THEN NULL
                        ELSE home.canonical_name || ' vs ' || away.canonical_name
                   END AS match_label,
                   (SELECT count(*) FROM ai_workspace.messages message
                    WHERE message.session_id = session.id)::bigint AS message_count,
                   (SELECT count(*) FROM ai_workspace.operation_proposals operation
                    WHERE operation.session_id = session.id AND operation.status = 'pending')::bigint
                    AS pending_operation_count
            FROM ai_workspace.sessions session
            LEFT JOIN football.matches fixture ON fixture.id = session.match_id
            LEFT JOIN football.teams home ON home.id = fixture.home_team_id
            LEFT JOIN football.teams away ON away.id = fixture.away_team_id
            WHERE session.id = $1
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("API协作会话不存在".to_string()))?;
        session_from_row(&row)
    }
}
