use super::DatabaseHealth;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::Utc;
use sqlx::Row;
use std::time::Instant;

impl PostgresStore {
    pub async fn health(&self) -> PersistenceResult<DatabaseHealth> {
        let started = Instant::now();
        let required_schema_ready: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM information_schema.columns
                WHERE table_schema = 'feature'
                  AND table_name = 'player_ability_observations'
                  AND column_name = 'created_at'
                  AND is_nullable = 'NO'
                  AND column_default IS NOT NULL
            )
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        if !required_schema_ready {
            return Err(PersistenceError::InvalidState(
                "数据库缺少当前客户端所需的能力观察写入时点字段".to_string(),
            ));
        }

        let row = sqlx::query(
            r#"
            SELECT
                current_database() AS database_name,
                current_setting('server_version') AS server_version,
                pg_database_size(current_database())::bigint AS database_size_bytes
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        let migration_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM _sqlx_migrations WHERE success = true",
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(DatabaseHealth {
            connected: true,
            database_name: row.try_get("database_name")?,
            server_version: row.try_get("server_version")?,
            migration_count,
            database_size_bytes: row.try_get("database_size_bytes")?,
            checked_at: Utc::now(),
            latency_ms: started.elapsed().as_millis(),
        })
    }
}
