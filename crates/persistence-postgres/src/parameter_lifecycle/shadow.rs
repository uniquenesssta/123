use super::{mapping::shadow_validation_from_row, policy::shadow_candidate_status};
use crate::{write_audit_event, PersistenceResult, PostgresStore};
use football_domain::ParameterShadowValidationRecord;
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn save_parameter_shadow_validation(
        &self,
        record: &ParameterShadowValidationRecord,
    ) -> PersistenceResult<ParameterShadowValidationRecord> {
        let candidate_status = shadow_candidate_status(&record.status);
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO analytics.parameter_shadow_validations (
                id, candidate_id, validation_key, partition_key, sample_count,
                baseline_metrics, candidate_metrics, metric_deltas, gate_results,
                status, generated_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            ON CONFLICT (candidate_id, validation_key) DO NOTHING
            "#,
        )
        .bind(record.id)
        .bind(record.candidate_id)
        .bind(&record.validation_key)
        .bind(&record.partition_key)
        .bind(record.sample_count as i64)
        .bind(&record.baseline_metrics)
        .bind(&record.candidate_metrics)
        .bind(&record.metric_deltas)
        .bind(&record.gate_results)
        .bind(&record.status)
        .bind(record.generated_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            UPDATE analytics.parameter_tuning_candidates
            SET status = $2,
                decided_at = now(),
                decision_note = CASE
                    WHEN $2 = 'shadow_passed' THEN '影子验证全部门禁通过，等待人工晋升'
                    WHEN $2 = 'shadow_failed' THEN '影子验证未通过，正式绑定保持不变'
                    ELSE '接入点 H 前置门禁未通过'
                END
            WHERE id = $1
              AND status IN ('accepted_for_backtest', 'shadow_running', 'blocked_by_h', 'shadow_failed')
            "#,
        )
        .bind(record.candidate_id)
        .bind(candidate_status)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "parameter_shadow_validation_recorded",
            "parameter_tuning_candidate",
            Some(record.candidate_id.to_string()),
            json!({
                "validation_id": record.id,
                "validation_key": record.validation_key,
                "status": record.status,
                "sample_count": record.sample_count,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_parameter_shadow_validation(record.candidate_id, &record.validation_key)
            .await
    }

    pub async fn list_parameter_shadow_validations(
        &self,
        candidate_id: Uuid,
    ) -> PersistenceResult<Vec<ParameterShadowValidationRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, candidate_id, validation_key, partition_key, sample_count,
                   baseline_metrics, candidate_metrics, metric_deltas, gate_results,
                   status, generated_at
            FROM analytics.parameter_shadow_validations
            WHERE candidate_id = $1
            ORDER BY generated_at DESC, id DESC
            "#,
        )
        .bind(candidate_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(shadow_validation_from_row).collect()
    }

    async fn read_parameter_shadow_validation(
        &self,
        candidate_id: Uuid,
        validation_key: &str,
    ) -> PersistenceResult<ParameterShadowValidationRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, candidate_id, validation_key, partition_key, sample_count,
                   baseline_metrics, candidate_metrics, metric_deltas, gate_results,
                   status, generated_at
            FROM analytics.parameter_shadow_validations
            WHERE candidate_id = $1 AND validation_key = $2
            "#,
        )
        .bind(candidate_id)
        .bind(validation_key)
        .fetch_one(&self.pool)
        .await?;
        shadow_validation_from_row(&row)
    }
}
