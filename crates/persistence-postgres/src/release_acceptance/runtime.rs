use crate::{PersistenceResult, PostgresStore};
use football_domain::ReleaseAcceptanceRuntimeFacts;
use sqlx::Row;

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
}
