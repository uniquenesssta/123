use super::{
    input::{evidence_claim_fingerprint, validate_evidence_claim},
    references::validate_evidence_version_references,
    row::evidence_claim_record_from_row,
};
use crate::adapters::p4::idempotency::{advisory_lock, ensure_idempotent_fingerprint};
use crate::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{EvidenceClaimDraft, EvidenceClaimRecord};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
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
}
