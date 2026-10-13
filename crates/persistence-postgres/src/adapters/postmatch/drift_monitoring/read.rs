use super::mapping::drift_finding_from_row;
use crate::{postmatch::non_negative_u64, PersistenceResult, PostgresStore};
use football_domain::PostmatchDriftRunRecord;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub(in crate::adapters::postmatch) async fn list_postmatch_drift_runs(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<PostmatchDriftRunRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT run.id, run.competition_id, competition.name AS competition_name,
                   run.competition_profile_id, run.model_version_id,
                   version.version AS model_version, run.parameter_set_id,
                   parameter.parameter_version, run.horizon, run.partition_key,
                   run.baseline_size, run.current_size, run.baseline_window,
                   run.current_window, run.status, run.run_key,
                   run.calculation_version, run.generated_at
            FROM analytics.postmatch_drift_runs run
            JOIN football.competitions competition ON competition.id=run.competition_id
            JOIN model.versions version ON version.id=run.model_version_id
            JOIN model.parameter_sets parameter ON parameter.id=run.parameter_set_id
            ORDER BY run.generated_at DESC, run.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        let mut output = Vec::with_capacity(rows.len());
        for row in rows {
            let run_id: Uuid = row.try_get("id")?;
            let finding_rows = sqlx::query(
                "SELECT metric_name,baseline_mean,current_mean,absolute_delta,relative_delta,severity,direction FROM analytics.postmatch_drift_findings WHERE run_id=$1 ORDER BY metric_name",
            )
            .bind(run_id)
            .fetch_all(&self.pool)
            .await?;
            let findings = finding_rows
                .iter()
                .map(drift_finding_from_row)
                .collect::<PersistenceResult<Vec<_>>>()?;
            output.push(PostmatchDriftRunRecord {
                id: run_id,
                competition_id: row.try_get("competition_id")?,
                competition_name: row.try_get("competition_name")?,
                competition_profile_id: row.try_get("competition_profile_id")?,
                model_version_id: row.try_get("model_version_id")?,
                model_version: row.try_get("model_version")?,
                parameter_set_id: row.try_get("parameter_set_id")?,
                parameter_version: row.try_get("parameter_version")?,
                horizon: row.try_get("horizon")?,
                partition_key: row.try_get("partition_key")?,
                baseline_size: non_negative_u64(row.try_get("baseline_size")?, "漂移基线样本")?,
                current_size: non_negative_u64(row.try_get("current_size")?, "漂移当前样本")?,
                baseline_window: row.try_get("baseline_window")?,
                current_window: row.try_get("current_window")?,
                status: row.try_get("status")?,
                run_key: row.try_get("run_key")?,
                calculation_version: row.try_get("calculation_version")?,
                findings,
                generated_at: row.try_get("generated_at")?,
            });
        }
        Ok(output)
    }
}
