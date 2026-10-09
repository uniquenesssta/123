use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use crate::adapters::p4::idempotency::{
    advisory_lock, ensure_idempotent_fingerprint, validate_idempotency_key,
};
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, P4Horizon, PromptVersionDraft,
    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
    ResearchRunStatus, SchemaVersionDraft, SchemaVersionRecord,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

impl PostgresStore {
    pub async fn register_schema_version(
        &self,
        draft: &SchemaVersionDraft,
    ) -> PersistenceResult<SchemaVersionRecord> {
        validate_version_identity(&draft.schema_key, &draft.version, "Schema")?;
        if draft.schema_kind.trim().is_empty() {
            return Err(PersistenceError::InvalidState(
                "schema_kind 不能为空".to_string(),
            ));
        }
        let content_sha256 = sha256_json(&draft.schema_body)?;
        let mut tx = self.pool.begin().await?;
        advisory_lock(
            &mut tx,
            &format!("schema:{}@{}", draft.schema_key, draft.version),
        )
        .await?;
        let record = if let Some(row) = sqlx::query(
            r#"
            SELECT id, schema_key, version, schema_kind, content_sha256, created_at
            FROM research.schema_versions
            WHERE schema_key = $1 AND version = $2
            "#,
        )
        .bind(&draft.schema_key)
        .bind(&draft.version)
        .fetch_optional(&mut *tx)
        .await?
        {
            ensure_same_hash(
                &draft.schema_key,
                &draft.version,
                row.try_get("content_sha256")?,
                &content_sha256,
            )?;
            ensure_same_identity_field(
                "Schema",
                &draft.schema_key,
                &draft.version,
                "schema_kind",
                row.try_get::<String, _>("schema_kind")?.as_str(),
                &draft.schema_kind,
            )?;
            schema_record_from_row(&row)?
        } else {
            let id = Uuid::new_v4();
            let row = sqlx::query(
                r#"
                INSERT INTO research.schema_versions (
                    id, schema_key, version, schema_kind, schema_body,
                    content_sha256, description, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING id, schema_key, version, schema_kind, content_sha256, created_at
                "#,
            )
            .bind(id)
            .bind(&draft.schema_key)
            .bind(&draft.version)
            .bind(&draft.schema_kind)
            .bind(&draft.schema_body)
            .bind(&content_sha256)
            .bind(&draft.description)
            .bind(&draft.metadata)
            .fetch_one(&mut *tx)
            .await?;
            write_audit_event(
                &mut tx,
                "schema_version_registered",
                "schema_version",
                Some(id.to_string()),
                json!({
                    "schema_key": draft.schema_key,
                    "version": draft.version,
                    "content_sha256": content_sha256,
                }),
            )
            .await?;
            schema_record_from_row(&row)?
        };
        tx.commit().await?;
        Ok(record)
    }

    pub async fn register_prompt_version(
        &self,
        draft: &PromptVersionDraft,
    ) -> PersistenceResult<PromptVersionRecord> {
        validate_version_identity(&draft.prompt_key, &draft.version, "Prompt")?;
        if draft.prompt_role.trim().is_empty() || draft.content.trim().is_empty() {
            return Err(PersistenceError::InvalidState(
                "prompt_role 和 prompt content 不能为空".to_string(),
            ));
        }
        let content_sha256 = sha256_bytes(draft.content.as_bytes());
        let mut tx = self.pool.begin().await?;
        advisory_lock(
            &mut tx,
            &format!("prompt:{}@{}", draft.prompt_key, draft.version),
        )
        .await?;
        let record = if let Some(row) = sqlx::query(
            r#"
            SELECT id, prompt_key, version, prompt_role, content_sha256, created_at
            FROM research.prompt_versions
            WHERE prompt_key = $1 AND version = $2
            "#,
        )
        .bind(&draft.prompt_key)
        .bind(&draft.version)
        .fetch_optional(&mut *tx)
        .await?
        {
            ensure_same_hash(
                &draft.prompt_key,
                &draft.version,
                row.try_get("content_sha256")?,
                &content_sha256,
            )?;
            ensure_same_identity_field(
                "Prompt",
                &draft.prompt_key,
                &draft.version,
                "prompt_role",
                row.try_get::<String, _>("prompt_role")?.as_str(),
                &draft.prompt_role,
            )?;
            prompt_record_from_row(&row)?
        } else {
            let id = Uuid::new_v4();
            let row = sqlx::query(
                r#"
                INSERT INTO research.prompt_versions (
                    id, prompt_key, version, prompt_role, content,
                    content_sha256, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, prompt_key, version, prompt_role, content_sha256, created_at
                "#,
            )
            .bind(id)
            .bind(&draft.prompt_key)
            .bind(&draft.version)
            .bind(&draft.prompt_role)
            .bind(&draft.content)
            .bind(&content_sha256)
            .bind(&draft.metadata)
            .fetch_one(&mut *tx)
            .await?;
            write_audit_event(
                &mut tx,
                "prompt_version_registered",
                "prompt_version",
                Some(id.to_string()),
                json!({
                    "prompt_key": draft.prompt_key,
                    "version": draft.version,
                    "content_sha256": content_sha256,
                }),
            )
            .await?;
            prompt_record_from_row(&row)?
        };
        tx.commit().await?;
        Ok(record)
    }

    pub async fn register_competition_profile_version(
        &self,
        draft: &CompetitionProfileVersionDraft,
    ) -> PersistenceResult<CompetitionProfileVersionRecord> {
        let mut tx = self.pool.begin().await?;
        let record = register_competition_profile_in_tx(&mut tx, draft).await?;
        tx.commit().await?;
        Ok(record)
    }

    pub async fn create_research_run(
        &self,
        draft: &ResearchRunDraft,
    ) -> PersistenceResult<ResearchRunRecord> {
        validate_idempotency_key(&draft.idempotency_key)?;
        let fingerprint = research_run_fingerprint(draft)?;
        let mut tx = self.pool.begin().await?;
        advisory_lock(&mut tx, &format!("research-run:{}", draft.idempotency_key)).await?;
        if let Some(row) = sqlx::query(
            r#"
            SELECT id, match_id, horizon, data_cutoff_at, trace_id, idempotency_key,
                   request_fingerprint, status, created_at
            FROM research.runs
            WHERE idempotency_key = $1
            "#,
        )
        .bind(&draft.idempotency_key)
        .fetch_optional(&mut *tx)
        .await?
        {
            let existing: String = row.try_get("request_fingerprint")?;
            ensure_idempotent_fingerprint(
                "研究任务",
                &draft.idempotency_key,
                &existing,
                &fingerprint,
            )?;
            let record = research_run_record_from_row(&row)?;
            tx.commit().await?;
            return Ok(record);
        }

        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO research.runs (
                id, match_id, horizon, data_cutoff_at, trace_id, idempotency_key,
                request_fingerprint, planner_version, prompt_version_id,
                schema_version_id, status, request_payload, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9,
                $10, 'planned', $11, $12
            )
            RETURNING id, match_id, horizon, data_cutoff_at, trace_id, idempotency_key,
                      request_fingerprint, status, created_at
            "#,
        )
        .bind(id)
        .bind(draft.match_id)
        .bind(draft.horizon.as_str())
        .bind(draft.data_cutoff_at)
        .bind(draft.trace_id)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .bind(&draft.planner_version)
        .bind(draft.prompt_version_id)
        .bind(draft.schema_version_id)
        .bind(&draft.request_payload)
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        let planned_payload = json!({"idempotency_key": draft.idempotency_key});
        let planned_fingerprint = sha256_json(&json!({
            "status": "planned",
            "payload": planned_payload,
        }))?;
        sqlx::query(
            r#"
            INSERT INTO research.run_events (
                id, research_run_id, status, payload, idempotency_key, event_fingerprint
            ) VALUES ($1, $2, 'planned', $3, 'created', $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(&planned_payload)
        .bind(&planned_fingerprint)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "research_run_created",
            "research_run",
            Some(id.to_string()),
            json!({
                "match_id": draft.match_id,
                "horizon": draft.horizon.as_str(),
                "data_cutoff_at": draft.data_cutoff_at,
                "trace_id": draft.trace_id,
                "request_fingerprint": fingerprint,
            }),
        )
        .await?;
        let record = research_run_record_from_row(&row)?;
        tx.commit().await?;
        Ok(record)
    }

    pub async fn record_research_run_event(
        &self,
        draft: &ResearchRunEventDraft,
    ) -> PersistenceResult<ResearchRunRecord> {
        validate_idempotency_key(&draft.idempotency_key)?;
        let event_fingerprint = sha256_json(&json!({
            "status": draft.status.as_str(),
            "response_id": draft.response_id,
            "model_id": draft.model_id,
            "token_usage": draft.token_usage,
            "error_category": draft.error_category,
            "error_message": draft.error_message,
            "payload": draft.payload,
        }))?;
        let mut tx = self.pool.begin().await?;
        advisory_lock(
            &mut tx,
            &format!(
                "research-run-event:{}:{}",
                draft.research_run_id, draft.idempotency_key
            ),
        )
        .await?;
        let inserted: Option<Uuid> = sqlx::query_scalar(
            r#"
            INSERT INTO research.run_events (
                id, research_run_id, status, response_id, model_id, token_usage,
                error_category, error_message, payload, idempotency_key, event_fingerprint
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (research_run_id, idempotency_key) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.research_run_id)
        .bind(draft.status.as_str())
        .bind(&draft.response_id)
        .bind(&draft.model_id)
        .bind(&draft.token_usage)
        .bind(&draft.error_category)
        .bind(&draft.error_message)
        .bind(&draft.payload)
        .bind(&draft.idempotency_key)
        .bind(&event_fingerprint)
        .fetch_optional(&mut *tx)
        .await?;

        if inserted.is_none() {
            let existing: String = sqlx::query_scalar(
                r#"
                SELECT event_fingerprint FROM research.run_events
                WHERE research_run_id = $1 AND idempotency_key = $2
                "#,
            )
            .bind(draft.research_run_id)
            .bind(&draft.idempotency_key)
            .fetch_one(&mut *tx)
            .await?;
            ensure_idempotent_fingerprint(
                "研究任务事件",
                &draft.idempotency_key,
                &existing,
                &event_fingerprint,
            )?;
        } else {
            let terminal = matches!(
                draft.status,
                ResearchRunStatus::Succeeded
                    | ResearchRunStatus::Partial
                    | ResearchRunStatus::Failed
                    | ResearchRunStatus::Cancelled
            );
            sqlx::query(
                r#"
                UPDATE research.runs
                SET status = $2,
                    response_id = COALESCE($3, response_id),
                    model_id = COALESCE($4, model_id),
                    token_usage = CASE WHEN $5 = '{}'::jsonb THEN token_usage ELSE $5 END,
                    error_category = $6,
                    error_message = $7,
                    attempt_count = attempt_count + CASE WHEN $2 = 'running' THEN 1 ELSE 0 END,
                    started_at = CASE WHEN $2 = 'running' THEN COALESCE(started_at, now()) ELSE started_at END,
                    finished_at = CASE WHEN $8 THEN now() ELSE NULL END,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(draft.research_run_id)
            .bind(draft.status.as_str())
            .bind(&draft.response_id)
            .bind(&draft.model_id)
            .bind(&draft.token_usage)
            .bind(&draft.error_category)
            .bind(&draft.error_message)
            .bind(terminal)
            .execute(&mut *tx)
            .await?;
        }

        let row = sqlx::query(
            r#"
            SELECT id, match_id, horizon, data_cutoff_at, trace_id, idempotency_key,
                   request_fingerprint, status, created_at
            FROM research.runs WHERE id = $1
            "#,
        )
        .bind(draft.research_run_id)
        .fetch_one(&mut *tx)
        .await?;
        let record = research_run_record_from_row(&row)?;
        tx.commit().await?;
        Ok(record)
    }
}

pub(crate) async fn register_competition_profile_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    draft: &CompetitionProfileVersionDraft,
) -> PersistenceResult<CompetitionProfileVersionRecord> {
    validate_version_identity(&draft.profile_key, &draft.version, "赛事Profile")?;
    if draft.name.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "赛事Profile名称不能为空".to_string(),
        ));
    }
    let definition_sha256 = sha256_json(&draft.definition)?;
    advisory_lock(
        tx,
        &format!("profile:{}@{}", draft.profile_key, draft.version),
    )
    .await?;
    if let Some(row) = sqlx::query(
        r#"
        SELECT id, profile_key, version, name, competition_kind,
               definition_sha256, created_at
        FROM model.competition_profiles
        WHERE profile_key = $1 AND version = $2
        "#,
    )
    .bind(&draft.profile_key)
    .bind(&draft.version)
    .fetch_optional(&mut **tx)
    .await?
    {
        ensure_same_hash(
            &draft.profile_key,
            &draft.version,
            row.try_get("definition_sha256")?,
            &definition_sha256,
        )?;
        ensure_same_identity_field(
            "赛事Profile",
            &draft.profile_key,
            &draft.version,
            "name",
            row.try_get::<String, _>("name")?.as_str(),
            &draft.name,
        )?;
        ensure_same_identity_field(
            "赛事Profile",
            &draft.profile_key,
            &draft.version,
            "competition_kind",
            row.try_get::<String, _>("competition_kind")?.as_str(),
            draft.competition_kind.as_str(),
        )?;
        return Ok(CompetitionProfileVersionRecord {
            id: row.try_get("id")?,
            profile_key: row.try_get("profile_key")?,
            version: row.try_get("version")?,
            definition_sha256: row.try_get("definition_sha256")?,
            created_at: row.try_get("created_at")?,
        });
    }
    let id = Uuid::new_v4();
    let row = sqlx::query(
        r#"
        INSERT INTO model.competition_profiles (
            id, profile_key, version, name, competition_kind,
            definition, definition_sha256, metadata
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, profile_key, version, definition_sha256, created_at
        "#,
    )
    .bind(id)
    .bind(&draft.profile_key)
    .bind(&draft.version)
    .bind(&draft.name)
    .bind(draft.competition_kind.as_str())
    .bind(&draft.definition)
    .bind(&definition_sha256)
    .bind(&draft.metadata)
    .fetch_one(&mut **tx)
    .await?;
    write_audit_event(
        tx,
        "competition_profile_registered",
        "competition_profile",
        Some(id.to_string()),
        json!({
            "profile_key": draft.profile_key,
            "version": draft.version,
            "definition_sha256": definition_sha256,
        }),
    )
    .await?;
    Ok(CompetitionProfileVersionRecord {
        id: row.try_get("id")?,
        profile_key: row.try_get("profile_key")?,
        version: row.try_get("version")?,
        definition_sha256: row.try_get("definition_sha256")?,
        created_at: row.try_get("created_at")?,
    })
}

fn research_run_fingerprint(draft: &ResearchRunDraft) -> PersistenceResult<String> {
    sha256_json(&json!({
        "match_id": draft.match_id,
        "horizon": draft.horizon.as_str(),
        "data_cutoff_at": draft.data_cutoff_at,
        "trace_id": draft.trace_id,
        "planner_version": draft.planner_version,
        "prompt_version_id": draft.prompt_version_id,
        "schema_version_id": draft.schema_version_id,
        "request_payload": draft.request_payload,
    }))
}

fn validate_version_identity(key: &str, version: &str, label: &str) -> PersistenceResult<()> {
    if key.trim().is_empty() || version.trim().is_empty() {
        return Err(PersistenceError::InvalidState(format!(
            "{label}键和版本不能为空"
        )));
    }
    Ok(())
}

fn ensure_same_identity_field(
    label: &str,
    key: &str,
    version: &str,
    field: &str,
    existing: &str,
    expected: &str,
) -> PersistenceResult<()> {
    if existing != expected {
        return Err(PersistenceError::InvalidState(format!(
            "{label} {key}@{version} 的不可变字段 {field} 已存在且内容不同"
        )));
    }
    Ok(())
}

fn ensure_same_hash(
    key: &str,
    version: &str,
    existing: String,
    expected: &str,
) -> PersistenceResult<()> {
    if existing != expected {
        return Err(PersistenceError::InvalidState(format!(
            "{key}@{version}已存在但内容指纹不同；必须发布新版本"
        )));
    }
    Ok(())
}

fn sha256_bytes(value: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value);
    hex::encode(hasher.finalize())
}

fn schema_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<SchemaVersionRecord> {
    Ok(SchemaVersionRecord {
        id: row.try_get("id")?,
        schema_key: row.try_get("schema_key")?,
        version: row.try_get("version")?,
        schema_kind: row.try_get("schema_kind")?,
        content_sha256: row.try_get("content_sha256")?,
        created_at: row.try_get("created_at")?,
    })
}

fn prompt_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PromptVersionRecord> {
    Ok(PromptVersionRecord {
        id: row.try_get("id")?,
        prompt_key: row.try_get("prompt_key")?,
        version: row.try_get("version")?,
        prompt_role: row.try_get("prompt_role")?,
        content_sha256: row.try_get("content_sha256")?,
        created_at: row.try_get("created_at")?,
    })
}

fn research_run_record_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ResearchRunRecord> {
    Ok(ResearchRunRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        horizon: parse_horizon(row.try_get::<String, _>("horizon")?.as_str())?,
        data_cutoff_at: row.try_get("data_cutoff_at")?,
        trace_id: row.try_get("trace_id")?,
        idempotency_key: row.try_get("idempotency_key")?,
        request_fingerprint: row.try_get("request_fingerprint")?,
        status: parse_research_status(row.try_get::<String, _>("status")?.as_str())?,
        created_at: row.try_get("created_at")?,
    })
}

pub(crate) fn parse_horizon(value: &str) -> PersistenceResult<P4Horizon> {
    match value {
        "T-24h" => Ok(P4Horizon::T24h),
        "T-6h" => Ok(P4Horizon::T6h),
        "T-90m" => Ok(P4Horizon::T90m),
        "T-1h" => Ok(P4Horizon::T1h),
        "T-N" => Ok(P4Horizon::LegacyTN),
        other => Err(PersistenceError::InvalidState(format!(
            "未知P4时点：{other}"
        ))),
    }
}

fn parse_research_status(value: &str) -> PersistenceResult<ResearchRunStatus> {
    match value {
        "planned" => Ok(ResearchRunStatus::Planned),
        "running" => Ok(ResearchRunStatus::Running),
        "succeeded" => Ok(ResearchRunStatus::Succeeded),
        "partial" => Ok(ResearchRunStatus::Partial),
        "failed" => Ok(ResearchRunStatus::Failed),
        "cancelled" => Ok(ResearchRunStatus::Cancelled),
        other => Err(PersistenceError::InvalidState(format!(
            "未知研究任务状态：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immutable_version_identity_rejects_semantic_field_drift() {
        assert!(ensure_same_identity_field(
            "Schema",
            "p4-evidence",
            "1.0.0",
            "schema_kind",
            "evidence",
            "snapshot",
        )
        .is_err());
        assert!(ensure_same_identity_field(
            "Prompt",
            "research",
            "1.0.0",
            "prompt_role",
            "research",
            "research",
        )
        .is_ok());
    }
}
