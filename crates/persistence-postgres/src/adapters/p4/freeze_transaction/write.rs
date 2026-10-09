use super::{
    details::write_snapshot_details,
    input::PreparedSnapshot,
    read::{existing_snapshot_by_idempotency, snapshot_record_by_id},
    validation::{validate_snapshot_evidence, validate_snapshot_references},
};
use crate::adapters::p4::idempotency::advisory_lock;
use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{PrematchSnapshotDraft, PrematchSnapshotRecord, SnapshotSourceKind};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
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
            RETURNING created_at, data_cutoff_time, frozen_at
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

        write_snapshot_details(&mut tx, id, &prepared).await?;

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
            data_cutoff_at: row.try_get("data_cutoff_time")?,
            frozen_at: row.try_get("frozen_at")?,
            snapshot_fingerprint: prepared.snapshot_fingerprint,
            idempotency_key: draft.idempotency_key.clone(),
            source_kind: draft.source_kind,
            evidence_scope: draft.source_kind.evidence_scope().to_string(),
            created: true,
            created_at,
        })
    }
}
