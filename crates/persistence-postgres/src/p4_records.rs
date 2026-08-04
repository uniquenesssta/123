use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, EvidenceClaimDraft,
    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, EvidenceVerificationState,
    P4Horizon, PrematchSnapshotBundle, PrematchSnapshotDraft, PrematchSnapshotRecord,
    PromptVersionDraft, PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft,
    ResearchRunRecord, ResearchRunStatus, SchemaVersionDraft, SchemaVersionRecord,
    SnapshotFeatureDraft, SnapshotProbabilityDraft, SnapshotSourceKind, P4_FEATURE_FIELD_COUNT,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{BTreeSet, HashSet};
use uuid::Uuid;

const PROBABILITY_TOLERANCE: f64 = 1e-9;

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

    pub async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PersistenceResult<EvidenceClaimRecord> {
        validate_evidence_claim(draft)?;
        let content_sha256 = sha256_json(&draft.value)?;
        let claim_fingerprint = evidence_claim_fingerprint(draft, &content_sha256)?;
        let mut tx = self.pool.begin().await?;
        advisory_lock(&mut tx, &format!("evidence:{}", draft.idempotency_key)).await?;
        if let Some(row) = sqlx::query(
            r#"
            SELECT id, match_id, field_key, verification_state, content_sha256,
                   claim_fingerprint, idempotency_key, created_at
            FROM research.evidence_claims
            WHERE idempotency_key = $1
            "#,
        )
        .bind(&draft.idempotency_key)
        .fetch_optional(&mut *tx)
        .await?
        {
            let existing: String = row.try_get("claim_fingerprint")?;
            ensure_idempotent_fingerprint(
                "证据声明",
                &draft.idempotency_key,
                &existing,
                &claim_fingerprint,
            )?;
            let record = evidence_claim_record_from_row(&row)?;
            tx.commit().await?;
            return Ok(record);
        }

        let run =
            sqlx::query("SELECT match_id, schema_version_id FROM research.runs WHERE id = $1")
                .bind(draft.research_run_id)
                .fetch_one(&mut *tx)
                .await?;
        if run.try_get::<Uuid, _>("match_id")? != draft.match_id {
            return Err(PersistenceError::InvalidState(
                "证据比赛与研究任务比赛不一致".to_string(),
            ));
        }
        if run.try_get::<Uuid, _>("schema_version_id")? != draft.schema_version_id {
            return Err(PersistenceError::InvalidState(
                "证据Schema版本与研究任务不一致".to_string(),
            ));
        }
        validate_evidence_version_references(&mut tx, draft).await?;

        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO research.evidence_claims (
                id, match_id, entity_type, entity_id, field_key, value,
                verification_state, source_tier, source_document_id,
                source_url, source_title, source_domain,
                published_at, observed_at, effective_at, retrieved_at, timezone,
                independent_source_count, conflict_group_id,
                content_sha256, claim_fingerprint, research_run_id,
                prompt_version_id, prompt_version, schema_version_id, schema_version,
                idempotency_key, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9,
                $10, $11, $12,
                $13, $14, $15, $16, $17,
                $18, $19,
                $20, $21, $22,
                $23, $24, $25, $26,
                $27, $28
            )
            RETURNING id, match_id, field_key, verification_state, content_sha256,
                      claim_fingerprint, idempotency_key, created_at
            "#,
        )
        .bind(id)
        .bind(draft.match_id)
        .bind(&draft.entity_type)
        .bind(draft.entity_id)
        .bind(&draft.field_key)
        .bind(&draft.value)
        .bind(draft.verification_state.as_str())
        .bind(&draft.source_tier)
        .bind(draft.source_document_id)
        .bind(&draft.source_url)
        .bind(&draft.source_title)
        .bind(&draft.source_domain)
        .bind(draft.published_at)
        .bind(draft.observed_at)
        .bind(draft.effective_at)
        .bind(draft.retrieved_at)
        .bind(&draft.timezone)
        .bind(i32::from(draft.independent_source_count))
        .bind(draft.conflict_group_id)
        .bind(&content_sha256)
        .bind(&claim_fingerprint)
        .bind(draft.research_run_id)
        .bind(draft.prompt_version_id)
        .bind(&draft.prompt_version)
        .bind(draft.schema_version_id)
        .bind(&draft.schema_version)
        .bind(&draft.idempotency_key)
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "evidence_claim_appended",
            "evidence_claim",
            Some(id.to_string()),
            json!({
                "match_id": draft.match_id,
                "field_key": draft.field_key,
                "verification_state": draft.verification_state.as_str(),
                "content_sha256": content_sha256,
                "claim_fingerprint": claim_fingerprint,
                "research_run_id": draft.research_run_id,
            }),
        )
        .await?;
        let record = evidence_claim_record_from_row(&row)?;
        tx.commit().await?;
        Ok(record)
    }

    pub async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PersistenceResult<EvidenceConflictRecord> {
        validate_idempotency_key(&draft.conflict_key)?;
        let evidence_ids = draft.evidence_ids.iter().copied().collect::<BTreeSet<_>>();
        if evidence_ids.len() < 2 {
            return Err(PersistenceError::InvalidState(
                "冲突组至少需要两条不同证据".to_string(),
            ));
        }
        let conflict_fingerprint = sha256_json(&json!({
            "match_id": draft.match_id,
            "entity_type": draft.entity_type,
            "entity_id": draft.entity_id,
            "field_key": draft.field_key,
            "evidence_ids": evidence_ids,
            "trace_id": draft.trace_id,
        }))?;
        let mut tx = self.pool.begin().await?;
        advisory_lock(&mut tx, &format!("conflict:{}", draft.conflict_key)).await?;
        if let Some(row) = sqlx::query(
            "SELECT id, conflict_key, conflict_fingerprint, created_at FROM research.evidence_conflicts WHERE conflict_key = $1",
        )
        .bind(&draft.conflict_key)
        .fetch_optional(&mut *tx)
        .await?
        {
            let existing: String = row.try_get("conflict_fingerprint")?;
            ensure_idempotent_fingerprint(
                "证据冲突",
                &draft.conflict_key,
                &existing,
                &conflict_fingerprint,
            )?;
            let record = EvidenceConflictRecord {
                id: row.try_get("id")?,
                conflict_key: row.try_get("conflict_key")?,
                created_at: row.try_get("created_at")?,
            };
            tx.commit().await?;
            return Ok(record);
        }

        let rows = sqlx::query(
            r#"
            SELECT id, match_id, entity_type, entity_id, field_key
            FROM research.evidence_claims
            WHERE id = ANY($1)
            "#,
        )
        .bind(evidence_ids.iter().copied().collect::<Vec<_>>())
        .fetch_all(&mut *tx)
        .await?;
        if rows.len() != evidence_ids.len() {
            return Err(PersistenceError::InvalidState(
                "冲突组包含不存在的证据".to_string(),
            ));
        }
        for row in &rows {
            let match_id: Uuid = row.try_get("match_id")?;
            let entity_type: String = row.try_get("entity_type")?;
            let entity_id: Option<Uuid> = row.try_get("entity_id")?;
            let field_key: String = row.try_get("field_key")?;
            if match_id != draft.match_id
                || entity_type != draft.entity_type
                || entity_id != draft.entity_id
                || field_key != draft.field_key
            {
                return Err(PersistenceError::InvalidState(
                    "冲突组证据必须属于同一比赛、实体和字段".to_string(),
                ));
            }
        }

        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO research.evidence_conflicts (
                id, match_id, entity_type, entity_id, field_key,
                conflict_key, conflict_fingerprint, trace_id, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, conflict_key, created_at
            "#,
        )
        .bind(id)
        .bind(draft.match_id)
        .bind(&draft.entity_type)
        .bind(draft.entity_id)
        .bind(&draft.field_key)
        .bind(&draft.conflict_key)
        .bind(&conflict_fingerprint)
        .bind(draft.trace_id)
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        for evidence_id in evidence_ids {
            sqlx::query(
                r#"
                INSERT INTO research.evidence_conflict_members (conflict_id, evidence_id)
                VALUES ($1, $2)
                "#,
            )
            .bind(id)
            .bind(evidence_id)
            .execute(&mut *tx)
            .await?;
        }
        let opened_payload = json!({"evidence_count": rows.len()});
        let opened_fingerprint = sha256_json(&json!({
            "event_type": "opened",
            "payload": opened_payload,
        }))?;
        sqlx::query(
            r#"
            INSERT INTO research.evidence_conflict_events (
                id, conflict_id, event_type, payload, idempotency_key, event_fingerprint
            ) VALUES ($1, $2, 'opened', $3, 'opened', $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(&opened_payload)
        .bind(&opened_fingerprint)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "evidence_conflict_opened",
            "evidence_conflict",
            Some(id.to_string()),
            json!({
                "match_id": draft.match_id,
                "field_key": draft.field_key,
                "conflict_key": draft.conflict_key,
                "evidence_count": rows.len(),
            }),
        )
        .await?;
        let record = EvidenceConflictRecord {
            id: row.try_get("id")?,
            conflict_key: row.try_get("conflict_key")?,
            created_at: row.try_get("created_at")?,
        };
        tx.commit().await?;
        Ok(record)
    }

    pub async fn freeze_prematch_snapshot(
        &self,
        draft: &PrematchSnapshotDraft,
    ) -> PersistenceResult<PrematchSnapshotRecord> {
        let prepared = PreparedSnapshot::new(draft)?;
        let mut tx = self.pool.begin().await?;
        advisory_lock(&mut tx, &format!("snapshot:{}", draft.idempotency_key)).await?;

        if let Some(record) = existing_snapshot_by_idempotency(
            &mut tx,
            &draft.idempotency_key,
            &prepared.snapshot_fingerprint,
        )
        .await?
        {
            tx.commit().await?;
            return Ok(record);
        }

        if matches!(
            draft.source_kind,
            SnapshotSourceKind::Real | SnapshotSourceKind::Manual
        ) {
            if let Some(row) = sqlx::query(
                r#"
                SELECT id, snapshot_fingerprint, idempotency_key
                FROM feature.snapshots
                WHERE match_id = $1
                  AND model_version_id = $2
                  AND parameter_set_id = $3
                  AND competition_profile_id = $4
                  AND snapshot_type = $5
                  AND data_cutoff_time = $6
                  AND source_kind IN ('real', 'manual')
                "#,
            )
            .bind(draft.match_id)
            .bind(draft.model_version_id)
            .bind(draft.parameter_set_id)
            .bind(draft.competition_profile_id)
            .bind(draft.horizon.as_str())
            .bind(draft.data_cutoff_at)
            .fetch_optional(&mut *tx)
            .await?
            {
                let existing: Option<String> = row.try_get("snapshot_fingerprint")?;
                let existing_key: Option<String> = row.try_get("idempotency_key")?;
                if existing.as_deref() != Some(&prepared.snapshot_fingerprint) {
                    return Err(PersistenceError::InvalidState(format!(
                        "精确队列已冻结不同载荷；现有快照 {}，幂等键 {}",
                        row.try_get::<Uuid, _>("id")?,
                        existing_key.unwrap_or_else(|| "未记录".to_string())
                    )));
                }
                let id: Uuid = row.try_get("id")?;
                let record = snapshot_record_by_id(&mut tx, id, false).await?;
                tx.commit().await?;
                return Ok(record);
            }
        }

        validate_snapshot_references(&mut tx, draft).await?;
        validate_snapshot_evidence(&mut tx, draft).await?;
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO feature.snapshots (
                id, match_id, match_key, snapshot_type, data_cutoff_time, frozen_at,
                schema_version, quality_score, input_payload, input_sha256,
                model_version_id, parameter_set_id, competition_profile_id,
                research_run_id, schema_version_id, trace_id, idempotency_key,
                snapshot_fingerprint, payload_sha256, feature_set_sha256,
                evidence_set_sha256, probability_set_sha256,
                source_kind, evidence_scope, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13,
                $14, $15, $16, $17,
                $18, $19, $20,
                $21, $22,
                $23, $24, $25
            )
            RETURNING created_at
            "#,
        )
        .bind(id)
        .bind(draft.match_id)
        .bind(&draft.match_key)
        .bind(draft.horizon.as_str())
        .bind(draft.data_cutoff_at)
        .bind(draft.frozen_at)
        .bind(&draft.schema_version)
        .bind(draft.quality_score)
        .bind(&draft.input_payload)
        .bind(&prepared.payload_sha256)
        .bind(draft.model_version_id)
        .bind(draft.parameter_set_id)
        .bind(draft.competition_profile_id)
        .bind(draft.research_run_id)
        .bind(draft.schema_version_id)
        .bind(draft.trace_id)
        .bind(&draft.idempotency_key)
        .bind(&prepared.snapshot_fingerprint)
        .bind(&prepared.payload_sha256)
        .bind(&prepared.feature_set_sha256)
        .bind(&prepared.evidence_set_sha256)
        .bind(&prepared.probability_set_sha256)
        .bind(draft.source_kind.as_str())
        .bind(draft.source_kind.evidence_scope())
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;

        for feature in &prepared.features {
            let value_sha256 = sha256_json(&feature.value)?;
            sqlx::query(
                r#"
                INSERT INTO feature.snapshot_features (
                    snapshot_id, field_order, field_key, value,
                    verification_state, evidence_ids, value_sha256, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(id)
            .bind(i16::from(feature.field_order))
            .bind(&feature.field_key)
            .bind(&feature.value)
            .bind(feature.verification_state.as_str())
            .bind(&feature.evidence_ids)
            .bind(value_sha256)
            .bind(&feature.metadata)
            .execute(&mut *tx)
            .await?;
            for evidence_id in &feature.evidence_ids {
                sqlx::query(
                    r#"
                    INSERT INTO feature.snapshot_evidence (snapshot_id, field_key, evidence_id)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(id)
                .bind(&feature.field_key)
                .bind(evidence_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        for probability in &prepared.probabilities {
            let is_formal = probability
                .metadata
                .get("formal")
                .and_then(Value::as_bool)
                .unwrap_or(probability.chain_key == "full");
            let shadow_status: Option<&str> = None;
            sqlx::query(
                r#"
                INSERT INTO model.snapshot_probabilities (
                    snapshot_id, chain_key, home_win, draw, away_win,
                    btts, over_2_5, clean_sheet_home, clean_sheet_away,
                    matrix_sha256, matrix_cell_count, is_formal, shadow_status, metadata
                ) VALUES (
                    $1, $2, $3, $4, $5,
                    $6, $7, $8, $9,
                    $10, $11, $12, $13, $14
                )
                "#,
            )
            .bind(id)
            .bind(&probability.chain_key)
            .bind(probability.home_win)
            .bind(probability.draw)
            .bind(probability.away_win)
            .bind(probability.btts)
            .bind(probability.over_2_5)
            .bind(probability.clean_sheet_home)
            .bind(probability.clean_sheet_away)
            .bind(&probability.matrix_sha256)
            .bind(i32::from(probability.matrix_cell_count))
            .bind(is_formal)
            .bind(shadow_status)
            .bind(&probability.metadata)
            .execute(&mut *tx)
            .await?;
        }

        write_audit_event(
            &mut tx,
            "prematch_snapshot_frozen",
            "prematch_snapshot",
            Some(id.to_string()),
            json!({
                "match_id": draft.match_id,
                "match_key": draft.match_key,
                "horizon": draft.horizon.as_str(),
                "model_version_id": draft.model_version_id,
                "parameter_set_id": draft.parameter_set_id,
                "competition_profile_id": draft.competition_profile_id,
                "snapshot_fingerprint": prepared.snapshot_fingerprint,
                "source_kind": draft.source_kind.as_str(),
                "evidence_scope": draft.source_kind.evidence_scope(),
                "trace_id": draft.trace_id,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(PrematchSnapshotRecord {
            id,
            match_id: draft.match_id,
            match_key: draft.match_key.clone(),
            horizon: draft.horizon,
            data_cutoff_at: draft.data_cutoff_at,
            frozen_at: draft.frozen_at,
            snapshot_fingerprint: prepared.snapshot_fingerprint,
            idempotency_key: draft.idempotency_key.clone(),
            source_kind: draft.source_kind,
            evidence_scope: draft.source_kind.evidence_scope().to_string(),
            created: true,
            created_at,
        })
    }

    pub async fn read_prematch_snapshot(
        &self,
        snapshot_id: Uuid,
    ) -> PersistenceResult<PrematchSnapshotBundle> {
        let mut tx = self.pool.begin().await?;
        let snapshot = snapshot_record_by_id(&mut tx, snapshot_id, false).await?;
        let input_payload: Value =
            sqlx::query_scalar("SELECT input_payload FROM feature.snapshots WHERE id = $1")
                .bind(snapshot_id)
                .fetch_one(&mut *tx)
                .await?;
        let feature_rows = sqlx::query(
            r#"
            SELECT field_order, field_key, value, verification_state, evidence_ids, metadata
            FROM feature.snapshot_features
            WHERE snapshot_id = $1
            ORDER BY field_order
            "#,
        )
        .bind(snapshot_id)
        .fetch_all(&mut *tx)
        .await?;
        let features = feature_rows
            .iter()
            .map(snapshot_feature_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        let probability_rows = sqlx::query(
            r#"
            SELECT chain_key, home_win, draw, away_win, btts, over_2_5,
                   clean_sheet_home, clean_sheet_away, matrix_sha256,
                   matrix_cell_count, metadata
            FROM model.snapshot_probabilities
            WHERE snapshot_id = $1
            ORDER BY chain_key
            "#,
        )
        .bind(snapshot_id)
        .fetch_all(&mut *tx)
        .await?;
        let probabilities = probability_rows
            .iter()
            .map(snapshot_probability_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        tx.commit().await?;
        Ok(PrematchSnapshotBundle {
            snapshot,
            input_payload,
            features,
            probabilities,
        })
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

struct PreparedSnapshot {
    payload_sha256: String,
    feature_set_sha256: String,
    evidence_set_sha256: String,
    probability_set_sha256: String,
    snapshot_fingerprint: String,
    features: Vec<SnapshotFeatureDraft>,
    probabilities: Vec<SnapshotProbabilityDraft>,
}

impl PreparedSnapshot {
    fn new(draft: &PrematchSnapshotDraft) -> PersistenceResult<Self> {
        validate_snapshot_draft(draft)?;
        let mut features = draft.features.clone();
        features.sort_by_key(|feature| feature.field_order);
        let mut probabilities = draft.probabilities.clone();
        probabilities.sort_by(|left, right| left.chain_key.cmp(&right.chain_key));
        let payload_sha256 = sha256_json(&draft.input_payload)?;
        let feature_set_sha256 = sha256_json(&features)?;
        let evidence_ids = features
            .iter()
            .flat_map(|feature| feature.evidence_ids.iter().copied())
            .collect::<BTreeSet<_>>();
        let evidence_set_sha256 = sha256_json(&evidence_ids)?;
        let probability_set_sha256 = sha256_json(&probabilities)?;
        let snapshot_fingerprint = sha256_json(&json!({
            "contract": football_domain::P4_PERSISTENCE_CONTRACT_VERSION,
            "match_id": draft.match_id,
            "match_key": draft.match_key,
            "horizon": draft.horizon.as_str(),
            "data_cutoff_at": draft.data_cutoff_at,
            "model_version_id": draft.model_version_id,
            "parameter_set_id": draft.parameter_set_id,
            "competition_profile_id": draft.competition_profile_id,
            "schema_version_id": draft.schema_version_id,
            "schema_version": draft.schema_version,
            "source_kind": draft.source_kind.as_str(),
            "evidence_scope": draft.source_kind.evidence_scope(),
            "payload_sha256": payload_sha256,
            "feature_set_sha256": feature_set_sha256,
            "evidence_set_sha256": evidence_set_sha256,
            "probability_set_sha256": probability_set_sha256,
        }))?;
        Ok(Self {
            payload_sha256,
            feature_set_sha256,
            evidence_set_sha256,
            probability_set_sha256,
            snapshot_fingerprint,
            features,
            probabilities,
        })
    }
}

async fn existing_snapshot_by_idempotency(
    tx: &mut Transaction<'_, Postgres>,
    idempotency_key: &str,
    expected_fingerprint: &str,
) -> PersistenceResult<Option<PrematchSnapshotRecord>> {
    let row = sqlx::query(
        r#"
        SELECT id, snapshot_fingerprint
        FROM feature.snapshots
        WHERE idempotency_key = $1
        "#,
    )
    .bind(idempotency_key)
    .fetch_optional(&mut **tx)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    let existing: Option<String> = row.try_get("snapshot_fingerprint")?;
    ensure_idempotent_fingerprint(
        "赛前快照",
        idempotency_key,
        existing.as_deref().unwrap_or_default(),
        expected_fingerprint,
    )?;
    let id: Uuid = row.try_get("id")?;
    snapshot_record_by_id(tx, id, false).await.map(Some)
}

async fn snapshot_record_by_id(
    tx: &mut Transaction<'_, Postgres>,
    id: Uuid,
    created: bool,
) -> PersistenceResult<PrematchSnapshotRecord> {
    let row = sqlx::query(
        r#"
        SELECT id, match_id, match_key, snapshot_type, data_cutoff_time, frozen_at,
               snapshot_fingerprint, idempotency_key, source_kind, evidence_scope, created_at
        FROM feature.snapshots
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&mut **tx)
    .await?;
    let match_id = row
        .try_get::<Option<Uuid>, _>("match_id")?
        .ok_or_else(|| PersistenceError::InvalidState("P4赛前快照缺少比赛关联".to_string()))?;
    Ok(PrematchSnapshotRecord {
        id: row.try_get("id")?,
        match_id,
        match_key: row.try_get("match_key")?,
        horizon: parse_horizon(row.try_get::<String, _>("snapshot_type")?.as_str())?,
        data_cutoff_at: row.try_get("data_cutoff_time")?,
        frozen_at: row.try_get("frozen_at")?,
        snapshot_fingerprint: row
            .try_get::<Option<String>, _>("snapshot_fingerprint")?
            .ok_or_else(|| PersistenceError::InvalidState("P4赛前快照缺少指纹".to_string()))?,
        idempotency_key: row
            .try_get::<Option<String>, _>("idempotency_key")?
            .ok_or_else(|| PersistenceError::InvalidState("P4赛前快照缺少幂等键".to_string()))?,
        source_kind: parse_source_kind(row.try_get::<String, _>("source_kind")?.as_str())?,
        evidence_scope: row.try_get("evidence_scope")?,
        created,
        created_at: row.try_get("created_at")?,
    })
}

async fn validate_evidence_version_references(
    tx: &mut Transaction<'_, Postgres>,
    draft: &EvidenceClaimDraft,
) -> PersistenceResult<()> {
    let registered_schema_version: Option<String> =
        sqlx::query_scalar("SELECT version FROM research.schema_versions WHERE id = $1")
            .bind(draft.schema_version_id)
            .fetch_optional(&mut **tx)
            .await?;
    if registered_schema_version.as_deref() != Some(draft.schema_version.as_str()) {
        return Err(PersistenceError::InvalidState(
            "证据Schema版本ID与版本号不一致".to_string(),
        ));
    }

    match (draft.prompt_version_id, draft.prompt_version.as_deref()) {
        (None, None) => {}
        (Some(prompt_version_id), Some(prompt_version)) => {
            let registered_prompt_version: Option<String> =
                sqlx::query_scalar("SELECT version FROM research.prompt_versions WHERE id = $1")
                    .bind(prompt_version_id)
                    .fetch_optional(&mut **tx)
                    .await?;
            if registered_prompt_version.as_deref() != Some(prompt_version) {
                return Err(PersistenceError::InvalidState(
                    "证据Prompt版本ID与版本号不一致".to_string(),
                ));
            }
        }
        _ => {
            return Err(PersistenceError::InvalidState(
                "证据Prompt版本ID与版本号必须同时提供或同时为空".to_string(),
            ));
        }
    }

    if let Some(conflict_group_id) = draft.conflict_group_id {
        let conflict = sqlx::query(
            r#"
            SELECT match_id, entity_type, entity_id, field_key
            FROM research.evidence_conflicts
            WHERE id = $1
            "#,
        )
        .bind(conflict_group_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("证据引用的冲突组不存在".to_string()))?;
        if conflict.try_get::<Uuid, _>("match_id")? != draft.match_id
            || conflict.try_get::<String, _>("entity_type")? != draft.entity_type
            || conflict.try_get::<Option<Uuid>, _>("entity_id")? != draft.entity_id
            || conflict.try_get::<String, _>("field_key")? != draft.field_key
        {
            return Err(PersistenceError::InvalidState(
                "证据引用的冲突组必须属于同一比赛、实体和字段".to_string(),
            ));
        }
    }
    Ok(())
}

async fn validate_snapshot_references(
    tx: &mut Transaction<'_, Postgres>,
    draft: &PrematchSnapshotDraft,
) -> PersistenceResult<()> {
    let match_row =
        sqlx::query("SELECT external_key, kickoff_time FROM football.matches WHERE id = $1")
            .bind(draft.match_id)
            .fetch_optional(&mut **tx)
            .await?
            .ok_or_else(|| {
                PersistenceError::InvalidState("赛前快照引用的比赛不存在".to_string())
            })?;
    let external_key: String = match_row.try_get("external_key")?;
    let kickoff_time: DateTime<Utc> = match_row.try_get("kickoff_time")?;
    if external_key != draft.match_key {
        return Err(PersistenceError::InvalidState(
            "match_key与比赛稳定外部键不一致".to_string(),
        ));
    }
    if draft.data_cutoff_at >= kickoff_time {
        return Err(PersistenceError::InvalidState(
            "赛前快照data_cutoff_at必须早于开球时间".to_string(),
        ));
    }

    let parameter_model_version: Option<Uuid> =
        sqlx::query_scalar("SELECT model_version_id FROM model.parameter_sets WHERE id = $1")
            .bind(draft.parameter_set_id)
            .fetch_optional(&mut **tx)
            .await?;
    if parameter_model_version != Some(draft.model_version_id) {
        return Err(PersistenceError::InvalidState(
            "参数版本不属于快照声明的模型版本".to_string(),
        ));
    }

    let profile_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM model.competition_profiles WHERE id = $1)")
            .bind(draft.competition_profile_id)
            .fetch_one(&mut **tx)
            .await?;
    if !profile_exists {
        return Err(PersistenceError::InvalidState(
            "赛前快照引用的赛事Profile版本不存在".to_string(),
        ));
    }

    let registered_schema_version: Option<String> =
        sqlx::query_scalar("SELECT version FROM research.schema_versions WHERE id = $1")
            .bind(draft.schema_version_id)
            .fetch_optional(&mut **tx)
            .await?;
    if registered_schema_version.as_deref() != Some(draft.schema_version.as_str()) {
        return Err(PersistenceError::InvalidState(
            "快照Schema版本ID与版本号不一致".to_string(),
        ));
    }

    if matches!(draft.source_kind, SnapshotSourceKind::Real) && draft.research_run_id.is_none() {
        return Err(PersistenceError::InvalidState(
            "真实联网快照必须关联研究任务".to_string(),
        ));
    }
    if let Some(research_run_id) = draft.research_run_id {
        let run = sqlx::query(
            r#"
            SELECT match_id, horizon, data_cutoff_at, trace_id
            FROM research.runs WHERE id = $1
            "#,
        )
        .bind(research_run_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("快照关联的研究任务不存在".to_string()))?;
        let run_horizon: String = run.try_get("horizon")?;
        if run.try_get::<Uuid, _>("match_id")? != draft.match_id
            || run_horizon != draft.horizon.as_str()
            || run.try_get::<DateTime<Utc>, _>("data_cutoff_at")? != draft.data_cutoff_at
            || run.try_get::<Uuid, _>("trace_id")? != draft.trace_id
        {
            return Err(PersistenceError::InvalidState(
                "研究任务与快照的比赛、时点、截止时间或追踪ID不一致".to_string(),
            ));
        }
    }
    Ok(())
}

async fn validate_snapshot_evidence(
    tx: &mut Transaction<'_, Postgres>,
    draft: &PrematchSnapshotDraft,
) -> PersistenceResult<()> {
    let evidence_ids = draft
        .features
        .iter()
        .flat_map(|feature| feature.evidence_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    if matches!(draft.source_kind, SnapshotSourceKind::SyntheticFixture) {
        if !evidence_ids.is_empty() {
            return Err(PersistenceError::InvalidState(
                "合成fixture不得链接真实证据声明".to_string(),
            ));
        }
        return Ok(());
    }
    if evidence_ids.is_empty() {
        return Ok(());
    }
    let rows = sqlx::query(
        r#"
        SELECT id, match_id, published_at, effective_at
        FROM research.evidence_claims
        WHERE id = ANY($1)
        "#,
    )
    .bind(evidence_ids.iter().copied().collect::<Vec<_>>())
    .fetch_all(&mut **tx)
    .await?;
    if rows.len() != evidence_ids.len() {
        return Err(PersistenceError::InvalidState(
            "快照包含不存在的证据声明".to_string(),
        ));
    }
    for row in rows {
        let match_id: Uuid = row.try_get("match_id")?;
        let published_at: Option<DateTime<Utc>> = row.try_get("published_at")?;
        let effective_at: Option<DateTime<Utc>> = row.try_get("effective_at")?;
        if match_id != draft.match_id {
            return Err(PersistenceError::InvalidState(
                "快照证据必须属于同一比赛".to_string(),
            ));
        }
        if published_at.is_some_and(|value| value > draft.data_cutoff_at)
            || effective_at.is_some_and(|value| value > draft.data_cutoff_at)
        {
            return Err(PersistenceError::InvalidState(
                "晚于data_cutoff_at的证据不得进入赛前快照".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_snapshot_draft(draft: &PrematchSnapshotDraft) -> PersistenceResult<()> {
    validate_idempotency_key(&draft.idempotency_key)?;
    if !draft.horizon.is_canonical() {
        return Err(PersistenceError::InvalidState(
            "P4计划冻结只接受T-24h、T-6h或T-1h；T-N由正式推演按需读取".to_string(),
        ));
    }
    if draft.match_key.trim().is_empty() || draft.schema_version.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "match_key和schema_version不能为空".to_string(),
        ));
    }
    if draft.frozen_at < draft.data_cutoff_at {
        return Err(PersistenceError::InvalidState(
            "frozen_at不能早于data_cutoff_at".to_string(),
        ));
    }
    if !draft.quality_score.is_finite() || !(0.0..=1.0).contains(&draft.quality_score) {
        return Err(PersistenceError::InvalidState(
            "quality_score必须在0..=1范围内".to_string(),
        ));
    }
    if draft.features.len() != P4_FEATURE_FIELD_COUNT {
        return Err(PersistenceError::InvalidState(format!(
            "P4赛前快照必须包含{P4_FEATURE_FIELD_COUNT}个A:AE语义字段，实际{}个",
            draft.features.len()
        )));
    }
    let orders = draft
        .features
        .iter()
        .map(|feature| feature.field_order)
        .collect::<BTreeSet<_>>();
    let expected_orders = (1..=P4_FEATURE_FIELD_COUNT as u8).collect::<BTreeSet<_>>();
    if orders != expected_orders {
        return Err(PersistenceError::InvalidState(
            "P4字段顺序必须完整覆盖1..=31且不得重复".to_string(),
        ));
    }
    let field_keys = draft
        .features
        .iter()
        .map(|feature| feature.field_key.trim())
        .collect::<HashSet<_>>();
    if field_keys.len() != P4_FEATURE_FIELD_COUNT || field_keys.contains("") {
        return Err(PersistenceError::InvalidState(
            "P4字段键不能为空或重复".to_string(),
        ));
    }
    let chains = draft
        .probabilities
        .iter()
        .map(|probability| probability.chain_key.trim())
        .collect::<BTreeSet<_>>();
    if chains.is_empty() || chains.contains("") || chains.len() != draft.probabilities.len() {
        return Err(PersistenceError::InvalidState(
            "冻结快照至少需要一条名称非空且不重复的外部模型概率链".to_string(),
        ));
    }
    for probability in &draft.probabilities {
        let values = [probability.home_win, probability.draw, probability.away_win];
        if values
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            || (values.iter().sum::<f64>() - 1.0).abs() > PROBABILITY_TOLERANCE
        {
            return Err(PersistenceError::InvalidState(format!(
                "{}链1X2概率必须有限、位于0..=1且和为1",
                probability.chain_key
            )));
        }
        if probability.matrix_cell_count == 0
            || probability.matrix_sha256.len() != 64
            || !probability
                .matrix_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(PersistenceError::InvalidState(format!(
                "{}链矩阵必须非空并带小写SHA-256",
                probability.chain_key
            )));
        }
        for optional in [
            probability.btts,
            probability.over_2_5,
            probability.clean_sheet_home,
            probability.clean_sheet_away,
        ]
        .into_iter()
        .flatten()
        {
            if !optional.is_finite() || !(0.0..=1.0).contains(&optional) {
                return Err(PersistenceError::InvalidState(
                    "概率扩展指标必须在0..=1范围内".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_evidence_claim(draft: &EvidenceClaimDraft) -> PersistenceResult<()> {
    validate_idempotency_key(&draft.idempotency_key)?;
    if draft.entity_type.trim().is_empty()
        || draft.field_key.trim().is_empty()
        || draft.source_tier.trim().is_empty()
        || draft.timezone.trim().is_empty()
        || draft.schema_version.trim().is_empty()
    {
        return Err(PersistenceError::InvalidState(
            "证据实体、字段、来源等级、时区和Schema版本不能为空".to_string(),
        ));
    }
    if draft.verification_state.requires_source()
        && (draft.source_url.as_deref().is_none_or(str::is_empty)
            || draft.source_title.as_deref().is_none_or(str::is_empty)
            || draft.source_domain.as_deref().is_none_or(str::is_empty))
    {
        return Err(PersistenceError::InvalidState(
            "有事实来源的证据必须保存URL、标题和域名".to_string(),
        ));
    }
    if draft.retrieved_at < draft.observed_at {
        return Err(PersistenceError::InvalidState(
            "retrieved_at不能早于observed_at".to_string(),
        ));
    }
    Ok(())
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

fn evidence_claim_fingerprint(
    draft: &EvidenceClaimDraft,
    content_sha256: &str,
) -> PersistenceResult<String> {
    sha256_json(&json!({
        "match_id": draft.match_id,
        "entity_type": draft.entity_type,
        "entity_id": draft.entity_id,
        "field_key": draft.field_key,
        "verification_state": draft.verification_state.as_str(),
        "source_tier": draft.source_tier,
        "source_document_id": draft.source_document_id,
        "source_url": draft.source_url,
        "source_title": draft.source_title,
        "source_domain": draft.source_domain,
        "published_at": draft.published_at,
        "observed_at": draft.observed_at,
        "effective_at": draft.effective_at,
        "retrieved_at": draft.retrieved_at,
        "timezone": draft.timezone,
        "independent_source_count": draft.independent_source_count,
        "conflict_group_id": draft.conflict_group_id,
        "content_sha256": content_sha256,
        "research_run_id": draft.research_run_id,
        "prompt_version_id": draft.prompt_version_id,
        "prompt_version": draft.prompt_version,
        "schema_version_id": draft.schema_version_id,
        "schema_version": draft.schema_version,
        "metadata": draft.metadata,
    }))
}

async fn advisory_lock(tx: &mut Transaction<'_, Postgres>, key: &str) -> PersistenceResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(key)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn validate_version_identity(key: &str, version: &str, label: &str) -> PersistenceResult<()> {
    if key.trim().is_empty() || version.trim().is_empty() {
        return Err(PersistenceError::InvalidState(format!(
            "{label}键和版本不能为空"
        )));
    }
    Ok(())
}

fn validate_idempotency_key(value: &str) -> PersistenceResult<()> {
    if value.trim().is_empty() || value.len() > 240 {
        return Err(PersistenceError::InvalidState(
            "幂等键不能为空且长度不得超过240".to_string(),
        ));
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

fn ensure_idempotent_fingerprint(
    entity: &str,
    idempotency_key: &str,
    existing: &str,
    expected: &str,
) -> PersistenceResult<()> {
    if existing != expected {
        return Err(PersistenceError::InvalidState(format!(
            "{entity}幂等键{idempotency_key}已绑定不同载荷"
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

fn evidence_claim_record_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EvidenceClaimRecord> {
    Ok(EvidenceClaimRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        field_key: row.try_get("field_key")?,
        verification_state: parse_verification_state(
            row.try_get::<String, _>("verification_state")?.as_str(),
        )?,
        content_sha256: row.try_get("content_sha256")?,
        claim_fingerprint: row.try_get("claim_fingerprint")?,
        idempotency_key: row.try_get("idempotency_key")?,
        created_at: row.try_get("created_at")?,
    })
}

fn snapshot_feature_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<SnapshotFeatureDraft> {
    let field_order: i16 = row.try_get("field_order")?;
    Ok(SnapshotFeatureDraft {
        field_order: u8::try_from(field_order)
            .map_err(|_| PersistenceError::InvalidState("快照字段顺序超出u8范围".to_string()))?,
        field_key: row.try_get("field_key")?,
        value: row.try_get("value")?,
        verification_state: parse_verification_state(
            row.try_get::<String, _>("verification_state")?.as_str(),
        )?,
        evidence_ids: row.try_get("evidence_ids")?,
        metadata: row.try_get("metadata")?,
    })
}

fn snapshot_probability_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<SnapshotProbabilityDraft> {
    let matrix_cell_count: i32 = row.try_get("matrix_cell_count")?;
    Ok(SnapshotProbabilityDraft {
        chain_key: row.try_get("chain_key")?,
        home_win: row.try_get("home_win")?,
        draw: row.try_get("draw")?,
        away_win: row.try_get("away_win")?,
        btts: row.try_get("btts")?,
        over_2_5: row.try_get("over_2_5")?,
        clean_sheet_home: row.try_get("clean_sheet_home")?,
        clean_sheet_away: row.try_get("clean_sheet_away")?,
        matrix_sha256: row.try_get("matrix_sha256")?,
        matrix_cell_count: u16::try_from(matrix_cell_count)
            .map_err(|_| PersistenceError::InvalidState("矩阵单元数超出u16范围".to_string()))?,
        metadata: row.try_get("metadata")?,
    })
}

fn parse_horizon(value: &str) -> PersistenceResult<P4Horizon> {
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

fn parse_source_kind(value: &str) -> PersistenceResult<SnapshotSourceKind> {
    match value {
        "real" => Ok(SnapshotSourceKind::Real),
        "manual" => Ok(SnapshotSourceKind::Manual),
        "synthetic_fixture" => Ok(SnapshotSourceKind::SyntheticFixture),
        other => Err(PersistenceError::InvalidState(format!(
            "记录不是P4正式快照来源：{other}"
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

fn parse_verification_state(value: &str) -> PersistenceResult<EvidenceVerificationState> {
    match value {
        "CONFIRMED" => Ok(EvidenceVerificationState::Confirmed),
        "PROBABLE" => Ok(EvidenceVerificationState::Probable),
        "CONFLICT" => Ok(EvidenceVerificationState::Conflict),
        "NOT_FOUND" => Ok(EvidenceVerificationState::NotFound),
        "STALE" => Ok(EvidenceVerificationState::Stale),
        "NOT_APPLICABLE" => Ok(EvidenceVerificationState::NotApplicable),
        other => Err(PersistenceError::InvalidState(format!(
            "未知证据验证状态：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn feature(order: u8) -> SnapshotFeatureDraft {
        SnapshotFeatureDraft {
            field_order: order,
            field_key: format!("field_{order:02}"),
            value: json!({"value": order}),
            verification_state: EvidenceVerificationState::Confirmed,
            evidence_ids: Vec::new(),
            metadata: json!({}),
        }
    }

    fn probability(chain: &str) -> SnapshotProbabilityDraft {
        SnapshotProbabilityDraft {
            chain_key: chain.to_string(),
            home_win: 0.47,
            draw: 0.30,
            away_win: 0.23,
            btts: Some(0.46),
            over_2_5: Some(0.42),
            clean_sheet_home: Some(0.38),
            clean_sheet_away: Some(0.23),
            matrix_sha256: "a".repeat(64),
            matrix_cell_count: 1,
            metadata: json!({}),
        }
    }

    fn snapshot() -> PrematchSnapshotDraft {
        let cutoff = Utc::now();
        PrematchSnapshotDraft {
            match_id: Uuid::new_v4(),
            match_key: "TEST-MATCH".to_string(),
            horizon: P4Horizon::T6h,
            data_cutoff_at: cutoff,
            frozen_at: cutoff + Duration::seconds(1),
            model_version_id: Uuid::new_v4(),
            parameter_set_id: Uuid::new_v4(),
            competition_profile_id: Uuid::new_v4(),
            research_run_id: None,
            schema_version_id: Uuid::new_v4(),
            schema_version: "snapshot-v1".to_string(),
            trace_id: Uuid::new_v4(),
            idempotency_key: "snapshot:test".to_string(),
            source_kind: SnapshotSourceKind::SyntheticFixture,
            quality_score: 1.0,
            input_payload: json!({"test": true}),
            features: (1..=31).map(feature).collect(),
            probabilities: ["primary", "secondary"]
                .iter()
                .map(|chain| probability(chain))
                .collect(),
            metadata: json!({}),
        }
    }

    #[test]
    fn snapshot_fingerprint_is_deterministic_and_order_independent() {
        let first = snapshot();
        let mut second = first.clone();
        second.features.reverse();
        second.probabilities.reverse();
        assert_eq!(
            PreparedSnapshot::new(&first).unwrap().snapshot_fingerprint,
            PreparedSnapshot::new(&second).unwrap().snapshot_fingerprint
        );
    }

    #[test]
    fn snapshot_requires_all_31_fields_and_at_least_one_chain() {
        let mut draft = snapshot();
        draft.features.pop();
        assert!(validate_snapshot_draft(&draft).is_err());

        let mut draft = snapshot();
        draft.probabilities.clear();
        assert!(validate_snapshot_draft(&draft).is_err());
    }

    #[test]
    fn duplicate_provider_chain_is_rejected() {
        let mut draft = snapshot();
        draft.probabilities[1].chain_key = draft.probabilities[0].chain_key.clone();
        assert!(validate_snapshot_draft(&draft).is_err());
    }

    #[test]
    fn evidence_source_is_required_for_supported_facts() {
        let now = Utc::now();
        let claim = EvidenceClaimDraft {
            match_id: Uuid::new_v4(),
            entity_type: "team".to_string(),
            entity_id: None,
            field_key: "injury".to_string(),
            value: json!({"status": "out"}),
            verification_state: EvidenceVerificationState::Confirmed,
            source_tier: "official".to_string(),
            source_document_id: None,
            source_url: None,
            source_title: None,
            source_domain: None,
            published_at: Some(now),
            observed_at: now,
            effective_at: Some(now),
            retrieved_at: now,
            timezone: "UTC".to_string(),
            independent_source_count: 1,
            conflict_group_id: None,
            research_run_id: Uuid::new_v4(),
            prompt_version_id: None,
            prompt_version: None,
            schema_version_id: Uuid::new_v4(),
            schema_version: "evidence-v1".to_string(),
            idempotency_key: "evidence:test".to_string(),
            metadata: json!({}),
        };
        assert!(validate_evidence_claim(&claim).is_err());
    }
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
