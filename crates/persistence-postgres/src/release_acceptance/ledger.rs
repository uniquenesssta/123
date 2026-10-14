use crate::{write_audit_event, PersistenceResult, PostgresStore};
use football_domain::{ReleaseAcceptanceCheck, ReleaseAcceptanceRun};
use serde_json::json;
use sqlx::Transaction;

impl PostgresStore {
    pub async fn persist_release_acceptance_run(
        &self,
        run: &ReleaseAcceptanceRun,
    ) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO platform.release_acceptance_runs (
              id, app_version, contract_version, fixture_version, overall_status,
              started_at, completed_at, requested_by, report_sha256,
              passed_count, warning_count, blocked_count,
              category_summaries, performance_summary, cost_summary, metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)
            "#,
        )
        .bind(run.id)
        .bind(&run.app_version)
        .bind(&run.contract_version)
        .bind(&run.fixture_version)
        .bind(run.overall_status.as_str())
        .bind(run.started_at)
        .bind(run.completed_at)
        .bind(run.requested_by.as_deref())
        .bind(&run.report_sha256)
        .bind(i32::try_from(run.passed_count).unwrap_or(i32::MAX))
        .bind(i32::try_from(run.warning_count).unwrap_or(i32::MAX))
        .bind(i32::try_from(run.blocked_count).unwrap_or(i32::MAX))
        .bind(serde_json::to_value(&run.category_summaries)?)
        .bind(serde_json::to_value(&run.performance)?)
        .bind(serde_json::to_value(&run.cost)?)
        .bind(json!({"acceptance_mode":"public_shell_and_runtime","provider_state":"NOT_BUNDLED"}))
        .execute(&mut *tx)
        .await?;

        for check in &run.checks {
            insert_release_acceptance_check(&mut tx, check).await?;
        }
        write_audit_event(
            &mut tx,
            "release_acceptance_completed",
            "release_acceptance_run",
            Some(run.id.to_string()),
            json!({
                "overall_status": run.overall_status.as_str(),
                "passed_count": run.passed_count,
                "warning_count": run.warning_count,
                "blocked_count": run.blocked_count,
                "report_sha256": run.report_sha256,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

async fn insert_release_acceptance_check(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    check: &ReleaseAcceptanceCheck,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
        INSERT INTO platform.release_acceptance_checks (
          id, run_id, sequence_no, category, check_code, title, status,
          summary, remediation, evidence, duration_ms
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        "#,
    )
    .bind(check.id)
    .bind(check.run_id)
    .bind(check.sequence_no)
    .bind(&check.category)
    .bind(&check.check_code)
    .bind(&check.title)
    .bind(check.status.as_str())
    .bind(&check.summary)
    .bind(check.remediation.as_deref())
    .bind(&check.evidence)
    .bind(check.duration_ms)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
