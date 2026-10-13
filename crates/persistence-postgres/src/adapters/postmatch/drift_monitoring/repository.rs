use super::super::monitoring::PostmatchPartition;
use super::calculation::prepare_drift;
use crate::{write_audit_event, PersistenceResult, PostgresStore};
use football_domain::POSTMATCH_MONITORING_VERSION;
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub(in crate::adapters::postmatch) async fn refresh_drift_for_partition(
        &self,
        partition: &PostmatchPartition,
        baseline_size: usize,
        current_size: usize,
    ) -> PersistenceResult<()> {
        let (samples, settlement_ids) = self.load_drift_samples(partition).await?;
        let prepared = prepare_drift(
            &samples,
            settlement_ids,
            &partition.partition_key,
            baseline_size,
            current_size,
        )?;
        let run_id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        let inserted = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO analytics.postmatch_drift_runs (
                id, competition_id, competition_profile_id, model_version_id,
                parameter_set_id, horizon, partition_key, baseline_size,
                current_size, baseline_window, current_window, status,
                run_key, calculation_version
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
            ON CONFLICT (run_key) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(run_id)
        .bind(partition.competition_id)
        .bind(partition.competition_profile_id)
        .bind(partition.model_version_id)
        .bind(partition.parameter_set_id)
        .bind(&partition.horizon)
        .bind(&partition.partition_key)
        .bind(prepared.baseline_count as i64)
        .bind(prepared.current_count as i64)
        .bind(&prepared.baseline_window)
        .bind(&prepared.current_window)
        .bind(prepared.status)
        .bind(&prepared.run_key)
        .bind(POSTMATCH_MONITORING_VERSION)
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(id) = inserted {
            for finding in &prepared.calculation.drift {
                sqlx::query(
                    r#"
                    INSERT INTO analytics.postmatch_drift_findings (
                        run_id, metric_name, baseline_mean, current_mean,
                        absolute_delta, relative_delta, severity, direction
                    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                    "#,
                )
                .bind(id)
                .bind(&finding.metric_name)
                .bind(finding.baseline_mean)
                .bind(finding.current_mean)
                .bind(finding.absolute_delta)
                .bind(finding.relative_delta)
                .bind(&finding.severity)
                .bind(&finding.direction)
                .execute(&mut *tx)
                .await?;
            }
            write_audit_event(
                &mut tx,
                "postmatch_drift_refreshed",
                "postmatch_drift_run",
                Some(id.to_string()),
                json!({
                    "partition_key": &partition.partition_key,
                    "status": prepared.status,
                    "baseline_size": prepared.baseline_count,
                    "current_size": prepared.current_count,
                }),
            )
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}
