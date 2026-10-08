use super::details::save_model_details;
use super::input::{prepared_feature_snapshot, prepared_run_input_audit};
use crate::mapping::{optional_uuid, to_json_value};
use crate::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::RouteDecision;
use football_model_api::{ModelOutput, ModelRequest};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn save_successful_run(
        &self,
        decision: &RouteDecision,
        request: &ModelRequest,
        output: &ModelOutput,
        duration_ms: i64,
    ) -> PersistenceResult<Uuid> {
        let run_id = Uuid::new_v4();
        let input_hash = sha256_json(&request.input)?;
        let input_audit = prepared_run_input_audit(&request.input)?;
        let summary = to_json_value(&output.summary)?;
        let database_match_id = optional_uuid(&request.input, "database_match_id")?;
        let feature_snapshot = prepared_feature_snapshot(&request.input, &request.snapshot_type)?;
        let mut feature_snapshot_id = feature_snapshot.as_ref().map(|snapshot| snapshot.id);
        if feature_snapshot.is_some() && database_match_id.is_none() {
            return Err(PersistenceError::InvalidState(
                "特征快照必须关联数据库比赛编号".to_string(),
            ));
        }
        let mut tx = self.pool.begin().await?;

        if let (Some(match_id), Some(snapshot)) = (database_match_id, feature_snapshot.as_ref()) {
            let inserted: Option<Uuid> = sqlx::query_scalar(
                r#"
                INSERT INTO feature.snapshots (
                    id, match_id, match_key, snapshot_type, data_cutoff_time,
                    frozen_at, schema_version, quality_score, input_payload, input_sha256,
                    snapshot_fingerprint, payload_sha256, source_kind, evidence_scope
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'runtime', 'none')
                ON CONFLICT DO NOTHING
                RETURNING id
                "#,
            )
            .bind(snapshot.id)
            .bind(match_id)
            .bind(&request.context.match_key)
            .bind(&request.snapshot_type)
            .bind(snapshot.data_cutoff_time)
            .bind(snapshot.frozen_at)
            .bind(&snapshot.schema_version)
            .bind(snapshot.quality_score)
            .bind(&request.input)
            .bind(&input_hash)
            .bind(&input_audit.manifest_sha256)
            .bind(&input_hash)
            .fetch_optional(&mut *tx)
            .await?;
            let persisted_id = if let Some(id) = inserted {
                id
            } else {
                sqlx::query_scalar(
                    r#"
                    SELECT id
                    FROM feature.snapshots
                    WHERE id = $1
                       OR (
                           match_key = $2 AND snapshot_type = $3 AND input_sha256 = $4
                           AND source_kind IN ('legacy', 'runtime')
                       )
                    ORDER BY CASE WHEN id = $1 THEN 0 ELSE 1 END
                    LIMIT 1
                    "#,
                )
                .bind(snapshot.id)
                .bind(&request.context.match_key)
                .bind(&request.snapshot_type)
                .bind(&input_hash)
                .fetch_one(&mut *tx)
                .await?
            };
            feature_snapshot_id = Some(persisted_id);
        }

        sqlx::query(
            r#"
            INSERT INTO model.runs (
                id, match_id, match_key, feature_snapshot_id,
                model_version_id, parameter_set_id,
                rule_package_id, route_binding_id, snapshot_type,
                route_reason, status, input_payload, output_payload, explanation,
                summary, input_sha256, duration_ms, completed_at,
                input_audit_version, input_readiness_level, input_readiness_score,
                input_manifest, input_manifest_sha256
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6,
                $7, $8, $9,
                $10, 'succeeded', $11, $12, $13,
                $14, $15, $16, now(),
                $17, $18, $19, $20, $21
            )
            "#,
        )
        .bind(run_id)
        .bind(database_match_id)
        .bind(&request.context.match_key)
        .bind(feature_snapshot_id)
        .bind(decision.model_version_id)
        .bind(decision.parameter_set_id)
        .bind(decision.rule_package_id)
        .bind(decision.binding_id)
        .bind(&request.snapshot_type)
        .bind(&decision.reason)
        .bind(&request.input)
        .bind(&output.payload)
        .bind(&output.explanation)
        .bind(summary)
        .bind(&input_hash)
        .bind(duration_ms)
        .bind(&input_audit.audit_version)
        .bind(&input_audit.readiness_level)
        .bind(input_audit.readiness_score)
        .bind(&input_audit.manifest)
        .bind(&input_audit.manifest_sha256)
        .execute(&mut *tx)
        .await?;

        save_model_details(&mut tx, run_id, &output.payload).await?;
        write_audit_event(
            &mut tx,
            "model_run_completed",
            "model_run",
            Some(run_id.to_string()),
            json!({
                "match_key": &request.context.match_key,
                "model": &request.identity,
                "rule_package_id": decision.rule_package_id,
                "binding_id": decision.binding_id,
                "duration_ms": duration_ms,
                "input_readiness_level": &input_audit.readiness_level,
                "input_readiness_score": input_audit.readiness_score,
                "input_manifest_sha256": &input_audit.manifest_sha256,
                "input_sha256": &input_hash,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(run_id)
    }
}
