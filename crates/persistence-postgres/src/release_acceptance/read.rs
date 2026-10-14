use super::mapping::{check_from_row, run_from_row, run_summary_from_row};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary};
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_release_acceptance_runs(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<ReleaseAcceptanceRunSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT id, app_version, overall_status, completed_at, requested_by,
                   passed_count, warning_count, blocked_count, report_sha256
            FROM platform.release_acceptance_runs
            ORDER BY completed_at DESC, id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(run_summary_from_row).collect()
    }

    pub async fn read_release_acceptance_run(
        &self,
        run_id: Uuid,
    ) -> PersistenceResult<ReleaseAcceptanceRun> {
        let row = sqlx::query(
            r#"
            SELECT id, app_version, contract_version, fixture_version, overall_status,
                   started_at, completed_at, requested_by, report_sha256,
                   passed_count, warning_count, blocked_count,
                   category_summaries, performance_summary, cost_summary
            FROM platform.release_acceptance_runs WHERE id=$1
            "#,
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("发布验收记录不存在".to_string()))?;
        let check_rows = sqlx::query(
            r#"
            SELECT id, run_id, sequence_no, category, check_code, title, status,
                   summary, remediation, evidence, duration_ms
            FROM platform.release_acceptance_checks
            WHERE run_id=$1 ORDER BY sequence_no
            "#,
        )
        .bind(run_id)
        .fetch_all(&self.pool)
        .await?;
        let checks = check_rows
            .iter()
            .map(check_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        run_from_row(&row, checks)
    }
}
