use super::input::prepared_conflict;
use crate::adapters::p4::idempotency::{advisory_lock, ensure_idempotent_fingerprint};
use crate::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{EvidenceConflictDraft, EvidenceConflictRecord};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PersistenceResult<EvidenceConflictRecord> {
        let (evidence_ids, conflict_fingerprint) = prepared_conflict(draft)?;
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
}
