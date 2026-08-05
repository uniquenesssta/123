use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ReleaseAcceptanceCheck, ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary,
    ReleaseAcceptanceRuntimeFacts, ReleaseAcceptanceStatus,
};
use serde_json::{json, Value};
use sqlx::{Row, Transaction};
use uuid::Uuid;

impl PostgresStore {
    pub async fn release_acceptance_runtime_facts(
        &self,
        performance_window_days: u32,
        cost_window_days: u32,
    ) -> PersistenceResult<ReleaseAcceptanceRuntimeFacts> {
        let health = self.health().await?;
        let stages = sqlx::query_scalar::<_, String>(
            "SELECT DISTINCT stage FROM platform.integration_contracts ORDER BY stage",
        )
        .fetch_all(&self.pool)
        .await?;
        let immutable_trigger_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)::bigint
            FROM pg_trigger trigger_row
            JOIN pg_class relation ON relation.oid = trigger_row.tgrelid
            JOIN pg_namespace namespace ON namespace.oid = relation.relnamespace
            WHERE NOT trigger_row.tgisinternal
              AND trigger_row.tgname LIKE '%immutable%'
              AND namespace.nspname IN ('platform','feature','research','review','analytics','model')
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let chain = sqlx::query(
            r#"
            SELECT
              (SELECT COUNT(*)::bigint FROM platform.integration_contracts WHERE contract_key='model-provider-boundary' AND metadata->>'provider_kind'='external' AND metadata->>'bundled_runtime'='false') AS provider_boundary_artifact_count,
              (SELECT COUNT(*)::bigint FROM platform.p4_freeze_tasks) AS freeze_task_count,
              (SELECT COUNT(*)::bigint FROM feature.snapshots WHERE frozen_at IS NOT NULL) AS frozen_snapshot_count,
              (SELECT COUNT(*)::bigint FROM review.postmatch_settlements) AS settlement_count,
              (SELECT COUNT(*)::bigint FROM review.evidence_scoring_decisions) AS evidence_decision_count,
              (SELECT COUNT(*)::bigint FROM analytics.parameter_shadow_validations) AS shadow_validation_count,
              (SELECT COUNT(*)::bigint FROM analytics.parameter_promotion_decisions) AS promotion_decision_count
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let performance = sqlx::query(
            r#"
            WITH recent_runs AS (
              SELECT status, duration_ms
              FROM model.runs
              WHERE created_at >= now() - ($1::int * interval '1 day')
            ), latest_query AS (
              SELECT tables
              FROM analytics.query_performance_snapshots
              ORDER BY captured_at DESC, id DESC
              LIMIT 1
            )
            SELECT
              COUNT(*)::bigint AS recent_model_run_count,
              percentile_cont(0.95) WITHIN GROUP (ORDER BY duration_ms)
                FILTER (WHERE status='succeeded' AND duration_ms IS NOT NULL) AS recent_model_run_p95_ms,
              COUNT(*) FILTER (WHERE status='failed')::bigint AS recent_model_failure_count,
              COALESCE((SELECT COUNT(*)::bigint FROM latest_query, jsonb_array_elements(latest_query.tables) item WHERE item->>'severity' IN ('warning','critical')),0)::bigint AS query_warning_count
            FROM recent_runs
            "#,
        )
        .bind(i32::try_from(performance_window_days.clamp(1, 365)).unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;

        let cost = sqlx::query(
            r#"
            WITH usage AS (
              SELECT * FROM research.openai_usage_daily
              WHERE usage_day_utc >= date_trunc('day', now() AT TIME ZONE 'UTC') - (($1::int - 1) * interval '1 day')
            ), latest_day AS (
              SELECT COALESCE(SUM(estimated_cost_usd),0)::double precision AS cost
              FROM usage
              WHERE usage_day_utc = (SELECT MAX(usage_day_utc) FROM usage)
            )
            SELECT
              COALESCE(SUM(completed_requests),0)::bigint AS completed_requests,
              COALESCE(SUM(failed_requests),0)::bigint AS failed_requests,
              COALESCE(SUM(search_calls),0)::bigint AS search_calls,
              COALESCE(SUM(estimated_cost_usd),0)::double precision AS estimated_cost_usd,
              COALESCE((SELECT cost FROM latest_day),0)::double precision AS latest_day_cost_usd
            FROM usage
            "#,
        )
        .bind(i32::try_from(cost_window_days.clamp(1, 365)).unwrap_or(30))
        .fetch_one(&self.pool)
        .await?;

        Ok(ReleaseAcceptanceRuntimeFacts {
            migration_count: health.migration_count,
            database_latency_ms: health.latency_ms,
            integration_stages: stages,
            immutable_trigger_count,
            provider_boundary_artifact_count: chain.try_get("provider_boundary_artifact_count")?,
            freeze_task_count: chain.try_get("freeze_task_count")?,
            frozen_snapshot_count: chain.try_get("frozen_snapshot_count")?,
            settlement_count: chain.try_get("settlement_count")?,
            evidence_decision_count: chain.try_get("evidence_decision_count")?,
            shadow_validation_count: chain.try_get("shadow_validation_count")?,
            promotion_decision_count: chain.try_get("promotion_decision_count")?,
            recent_model_run_count: performance.try_get("recent_model_run_count")?,
            recent_model_run_p95_ms: performance.try_get("recent_model_run_p95_ms")?,
            recent_model_failure_count: performance.try_get("recent_model_failure_count")?,
            query_warning_count: performance.try_get("query_warning_count")?,
            completed_requests: cost.try_get("completed_requests")?,
            failed_requests: cost.try_get("failed_requests")?,
            search_calls: cost.try_get("search_calls")?,
            estimated_cost_usd: cost.try_get("estimated_cost_usd")?,
            latest_day_cost_usd: cost.try_get("latest_day_cost_usd")?,
        })
    }

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
        Ok(ReleaseAcceptanceRun {
            id: row.try_get("id")?,
            app_version: row.try_get("app_version")?,
            contract_version: row.try_get("contract_version")?,
            fixture_version: row.try_get("fixture_version")?,
            overall_status: parse_status(row.try_get::<String, _>("overall_status")?.as_str())?,
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            requested_by: row.try_get("requested_by")?,
            report_sha256: row.try_get("report_sha256")?,
            passed_count: u32::try_from(row.try_get::<i32, _>("passed_count")?).unwrap_or_default(),
            warning_count: u32::try_from(row.try_get::<i32, _>("warning_count")?)
                .unwrap_or_default(),
            blocked_count: u32::try_from(row.try_get::<i32, _>("blocked_count")?)
                .unwrap_or_default(),
            category_summaries: serde_json::from_value(
                row.try_get::<Value, _>("category_summaries")?,
            )?,
            performance: serde_json::from_value(row.try_get::<Value, _>("performance_summary")?)?,
            cost: serde_json::from_value(row.try_get::<Value, _>("cost_summary")?)?,
            checks,
        })
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

fn parse_status(value: &str) -> PersistenceResult<ReleaseAcceptanceStatus> {
    match value {
        "pass" => Ok(ReleaseAcceptanceStatus::Pass),
        "warning" => Ok(ReleaseAcceptanceStatus::Warning),
        "blocked" => Ok(ReleaseAcceptanceStatus::Blocked),
        other => Err(PersistenceError::InvalidState(format!(
            "未知发布验收状态：{other}"
        ))),
    }
}

fn run_summary_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ReleaseAcceptanceRunSummary> {
    Ok(ReleaseAcceptanceRunSummary {
        id: row.try_get("id")?,
        app_version: row.try_get("app_version")?,
        overall_status: parse_status(row.try_get::<String, _>("overall_status")?.as_str())?,
        completed_at: row.try_get("completed_at")?,
        requested_by: row.try_get("requested_by")?,
        passed_count: u32::try_from(row.try_get::<i32, _>("passed_count")?).unwrap_or_default(),
        warning_count: u32::try_from(row.try_get::<i32, _>("warning_count")?).unwrap_or_default(),
        blocked_count: u32::try_from(row.try_get::<i32, _>("blocked_count")?).unwrap_or_default(),
        report_sha256: row.try_get("report_sha256")?,
    })
}

fn check_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<ReleaseAcceptanceCheck> {
    Ok(ReleaseAcceptanceCheck {
        id: row.try_get("id")?,
        run_id: row.try_get("run_id")?,
        sequence_no: row.try_get("sequence_no")?,
        category: row.try_get("category")?,
        check_code: row.try_get("check_code")?,
        title: row.try_get("title")?,
        status: parse_status(row.try_get::<String, _>("status")?.as_str())?,
        summary: row.try_get("summary")?,
        remediation: row.try_get("remediation")?,
        evidence: row.try_get("evidence")?,
        duration_ms: row.try_get("duration_ms")?,
    })
}
