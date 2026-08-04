use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ApiWorkspaceGeneratedFileContent, ApiWorkspaceGeneratedFileDraft,
    ApiWorkspaceGeneratedFileRecord, ApiWorkspaceMessageDraft, ApiWorkspaceMessageRecord,
    ApiWorkspaceOperationDraft, ApiWorkspaceOperationRecord, ApiWorkspaceSessionDetail,
    ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord, OpenAiUsageTotals,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn api_workspace_usage_totals(&self) -> PersistenceResult<OpenAiUsageTotals> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(sum(
                    COALESCE(NULLIF(token_usage->>'estimated_cost_usd', '')::float8, 0)
                ) FILTER (
                    WHERE created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                ), 0)::float8 AS today_cost_usd,
                COALESCE(sum(
                    COALESCE(NULLIF(token_usage->>'estimated_cost_usd', '')::float8, 0)
                ) FILTER (
                    WHERE created_at >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                ), 0)::float8 AS month_cost_usd,
                count(*) FILTER (
                    WHERE created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                )::bigint AS today_request_count,
                count(*) FILTER (
                    WHERE created_at >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                )::bigint AS month_request_count
            FROM ai_workspace.messages
            WHERE role = 'assistant'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(OpenAiUsageTotals {
            today_cost_usd: row.try_get("today_cost_usd")?,
            month_cost_usd: row.try_get("month_cost_usd")?,
            today_request_count: u64::try_from(row.try_get::<i64, _>("today_request_count")?)
                .map_err(|_| {
                    PersistenceError::InvalidState("今日API工作台请求数为负数".to_string())
                })?,
            month_request_count: u64::try_from(row.try_get::<i64, _>("month_request_count")?)
                .map_err(|_| {
                    PersistenceError::InvalidState("本月API工作台请求数为负数".to_string())
                })?,
        })
    }

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

    pub async fn read_api_workspace_generated_file(
        &self,
        file_id: Uuid,
    ) -> PersistenceResult<ApiWorkspaceGeneratedFileContent> {
        let row = sqlx::query(
            r#"
            SELECT id, session_id, message_id, filename, media_type, content_sha256,
                   octet_length(content)::bigint AS size_bytes, content, created_at
            FROM ai_workspace.generated_files
            WHERE id = $1
            "#,
        )
        .bind(file_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("API生成文件不存在".to_string()))?;
        Ok(ApiWorkspaceGeneratedFileContent {
            file: file_from_row(&row)?,
            content: row.try_get("content")?,
        })
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

fn validate_message_draft(draft: &ApiWorkspaceMessageDraft) -> PersistenceResult<()> {
    if !matches!(draft.role.as_str(), "user" | "assistant" | "system") {
        return Err(PersistenceError::InvalidState(
            "API协作消息角色无效".to_string(),
        ));
    }
    if draft.content.chars().count() > 100_000 {
        return Err(PersistenceError::InvalidState(
            "API协作消息超过100000字符".to_string(),
        ));
    }
    Ok(())
}

fn validate_operation_draft(draft: &ApiWorkspaceOperationDraft) -> PersistenceResult<()> {
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

fn validate_generated_file_draft(draft: &ApiWorkspaceGeneratedFileDraft) -> PersistenceResult<()> {
    if draft.filename.trim().is_empty()
        || draft.filename.chars().count() > 120
        || draft.filename.chars().any(|ch| "\\/:*?\"<>|".contains(ch))
        || !matches!(
            draft.media_type.as_str(),
            "text/plain" | "text/markdown" | "application/json" | "text/csv"
        )
        || draft.content.len() > 2 * 1024 * 1024
    {
        return Err(PersistenceError::InvalidState(
            "API生成文件名称、类型或大小无效".to_string(),
        ));
    }
    Ok(())
}

fn session_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<ApiWorkspaceSessionRecord> {
    Ok(ApiWorkspaceSessionRecord {
        id: row.try_get("id")?,
        profile_id: row.try_get("profile_id")?,
        preset_key: row.try_get("preset_key")?,
        title: row.try_get("title")?,
        match_id: row.try_get("match_id")?,
        match_label: row.try_get("match_label")?,
        metadata: row.try_get("metadata")?,
        status: row.try_get("status")?,
        message_count: row.try_get("message_count")?,
        pending_operation_count: row.try_get("pending_operation_count")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn message_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<ApiWorkspaceMessageRecord> {
    Ok(ApiWorkspaceMessageRecord {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        role: row.try_get("role")?,
        content: row.try_get("content")?,
        structured_payload: row.try_get("structured_payload")?,
        citations: row.try_get("citations")?,
        attachments: row.try_get("attachments")?,
        provider_response_id: row.try_get("provider_response_id")?,
        model_id: row.try_get("model_id")?,
        token_usage: row.try_get("token_usage")?,
        created_at: row.try_get("created_at")?,
    })
}

fn operation_from_row(
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

fn file_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ApiWorkspaceGeneratedFileRecord> {
    Ok(ApiWorkspaceGeneratedFileRecord {
        id: row.try_get("id")?,
        session_id: row.try_get("session_id")?,
        message_id: row.try_get("message_id")?,
        filename: row.try_get("filename")?,
        media_type: row.try_get("media_type")?,
        content_sha256: row.try_get("content_sha256")?,
        size_bytes: row.try_get("size_bytes")?,
        created_at: row.try_get("created_at")?,
    })
}
