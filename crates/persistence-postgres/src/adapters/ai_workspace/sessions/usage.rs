use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::OpenAiUsageTotals;
use sqlx::Row;

impl PostgresStore {
    pub async fn api_workspace_usage_totals(&self) -> PersistenceResult<OpenAiUsageTotals> {
        let row = sqlx::query(
            r#"
            SELECT
                COALESCE(sum(
                    COALESCE(NULLIF(token_usage->>'estimated_cost_usd', '')::float8, 0)
                ) FILTER (
                    WHERE created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                ), 0)::float8 AS today_cost_usd,
                COALESCE(sum(
                    COALESCE(NULLIF(token_usage->>'estimated_cost_usd', '')::float8, 0)
                ) FILTER (
                    WHERE created_at >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                ), 0)::float8 AS month_cost_usd,
                count(*) FILTER (
                    WHERE created_at >= date_trunc('day', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                )::bigint AS today_request_count,
                count(*) FILTER (
                    WHERE created_at >= date_trunc('month', now() AT TIME ZONE 'UTC') AT TIME ZONE 'UTC'
                )::bigint AS month_request_count
            FROM ai_workspace.messages
            WHERE role = 'assistant'
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(OpenAiUsageTotals {
            today_cost_usd: row.try_get("today_cost_usd")?,
            month_cost_usd: row.try_get("month_cost_usd")?,
            today_request_count: u64::try_from(row.try_get::<i64, _>("today_request_count")?)
                .map_err(|_| {
                    PersistenceError::InvalidState("今日API工作台请求数为负数".to_string())
                })?,
            month_request_count: u64::try_from(row.try_get::<i64, _>("month_request_count")?)
                .map_err(|_| {
                    PersistenceError::InvalidState("本月API工作台请求数为负数".to_string())
                })?,
        })
    }
}
