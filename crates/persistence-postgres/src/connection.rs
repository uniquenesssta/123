use super::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::time::{Duration, Instant};

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

const APPLICATION_SCHEMAS: [&str; 10] = [
    "ai_workspace",
    "analytics",
    "audit",
    "catalog",
    "feature",
    "football",
    "model",
    "platform",
    "research",
    "review",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOptions {
    pub connection_url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

fn default_max_connections() -> u32 {
    10
}

fn default_connect_timeout_seconds() -> u64 {
    10
}

impl DatabaseOptions {
    pub fn redacted_url(&self) -> String {
        redact_database_url(&self.connection_url)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub connected: bool,
    pub database_name: String,
    pub server_version: String,
    pub migration_count: i64,
    pub database_size_bytes: i64,
    pub checked_at: DateTime<Utc>,
    pub latency_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub competitions: i64,
    pub teams: i64,
    pub players: i64,
    pub matches: i64,
    pub model_runs: i64,
    pub rule_packages: i64,
    pub route_bindings: i64,
    pub ability_observations: i64,
    pub pending_ability_updates: i64,
    pub data_providers: i64,
    pub availability_records: i64,
    pub active_lineups: i64,
    pub large_counts_are_estimates: bool,
}

impl PostgresStore {
    pub async fn connect(options: &DatabaseOptions) -> PersistenceResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(options.max_connections.max(1))
            .acquire_timeout(Duration::from_secs(options.connect_timeout_seconds.max(1)))
            .connect(&options.connection_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> PersistenceResult<()> {
        MIGRATOR.run(&self.pool).await?;
        self.ensure_runtime_schema_compatibility().await?;
        sqlx::query_scalar::<_, i64>("SELECT feature.refresh_player_ability_projections()")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn reset_to_pristine(&self) -> PersistenceResult<()> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "SELECT pg_advisory_xact_lock(hashtext('football-platform-destructive-reset')::bigint)",
        )
        .execute(&mut *transaction)
        .await?;

        for schema in APPLICATION_SCHEMAS {
            let statement = format!("DROP SCHEMA IF EXISTS {schema} CASCADE");
            sqlx::query(&statement)
                .execute(&mut *transaction)
                .await?;
        }
        sqlx::query("DROP TABLE IF EXISTS public._sqlx_migrations")
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;

        self.migrate().await
    }

    async fn ensure_runtime_schema_compatibility(&self) -> PersistenceResult<()> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query(
            "SELECT pg_advisory_xact_lock(hashtext('football-platform-schema-compatibility')::bigint)",
        )
        .execute(&mut *transaction)
        .await?;

        let observation_table_exists: bool = sqlx::query_scalar(
            "SELECT to_regclass('feature.player_ability_observations') IS NOT NULL",
        )
        .fetch_one(&mut *transaction)
        .await?;
        if !observation_table_exists {
            transaction.commit().await?;
            return Ok(());
        }

        let cutoff_column_ready: bool = sqlx::query_scalar(
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
        .fetch_one(&mut *transaction)
        .await?;

        if !cutoff_column_ready {
            sqlx::query(
                "ALTER TABLE feature.player_ability_observations ADD COLUMN IF NOT EXISTS created_at timestamptz",
            )
            .execute(&mut *transaction)
            .await?;
            sqlx::query(
                "UPDATE feature.player_ability_observations SET created_at = observed_at WHERE created_at IS NULL",
            )
            .execute(&mut *transaction)
            .await?;
            sqlx::query(
                "ALTER TABLE feature.player_ability_observations ALTER COLUMN created_at SET DEFAULT now()",
            )
            .execute(&mut *transaction)
            .await?;
            sqlx::query(
                "ALTER TABLE feature.player_ability_observations ALTER COLUMN created_at SET NOT NULL",
            )
            .execute(&mut *transaction)
            .await?;
        }

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS ability_observations_player_cutoff_idx
                ON feature.player_ability_observations
                   (player_id, created_at DESC, dimension_code, effective_from DESC)
            "#,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }

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

    pub async fn stats(&self) -> PersistenceResult<DatabaseStats> {
        // 巨型事实表使用 PostgreSQL 统计估算，避免每次启动都执行 COUNT(*) 全表扫描。
        // 小型配置表和带状态条件的工作表仍返回精确数量。
        let row = sqlx::query(
            r#"
            WITH table_estimates AS (
                SELECT
                    COALESCE(max(n_live_tup) FILTER (
                        WHERE schemaname = 'football' AND relname = 'teams'
                    ), 0)::bigint AS teams,
                    COALESCE(max(n_live_tup) FILTER (
                        WHERE schemaname = 'football' AND relname = 'players'
                    ), 0)::bigint AS players,
                    COALESCE(max(n_live_tup) FILTER (
                        WHERE schemaname = 'football' AND relname = 'matches'
                    ), 0)::bigint AS matches,
                    COALESCE(max(n_live_tup) FILTER (
                        WHERE schemaname = 'model' AND relname = 'runs'
                    ), 0)::bigint AS model_runs,
                    COALESCE(max(n_live_tup) FILTER (
                        WHERE schemaname = 'feature' AND relname = 'player_ability_observations'
                    ), 0)::bigint AS ability_observations,
                    COALESCE(max(n_live_tup) FILTER (
                        WHERE schemaname = 'football' AND relname = 'player_availability'
                    ), 0)::bigint AS availability_records
                FROM pg_stat_user_tables
            )
            SELECT
                (SELECT COUNT(*)::bigint FROM football.competitions) AS competitions,
                estimate.teams,
                estimate.players,
                estimate.matches,
                estimate.model_runs,
                (SELECT COUNT(*)::bigint FROM model.rule_packages) AS rule_packages,
                (SELECT COUNT(*)::bigint FROM model.competition_bindings WHERE is_active) AS route_bindings,
                estimate.ability_observations,
                (SELECT COUNT(*)::bigint FROM review.ability_update_candidates WHERE status = 'pending') AS pending_ability_updates,
                (SELECT COUNT(*)::bigint FROM catalog.data_providers WHERE is_active) AS data_providers,
                estimate.availability_records,
                (SELECT COUNT(*)::bigint FROM football.lineups WHERE status = 'active') AS active_lineups
            FROM table_estimates estimate
            "#,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(DatabaseStats {
            competitions: row.try_get("competitions")?,
            teams: row.try_get("teams")?,
            players: row.try_get("players")?,
            matches: row.try_get("matches")?,
            model_runs: row.try_get("model_runs")?,
            rule_packages: row.try_get("rule_packages")?,
            route_bindings: row.try_get("route_bindings")?,
            ability_observations: row.try_get("ability_observations")?,
            pending_ability_updates: row.try_get("pending_ability_updates")?,
            data_providers: row.try_get("data_providers")?,
            availability_records: row.try_get("availability_records")?,
            active_lineups: row.try_get("active_lineups")?,
            large_counts_are_estimates: true,
        })
    }
}

fn redact_database_url(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return "已配置".to_string();
    };
    let scheme = &url[..scheme_end + 3];
    let rest = &url[scheme_end + 3..];
    let Some(at) = rest.rfind('@') else {
        return url.to_string();
    };
    let credentials = &rest[..at];
    let host = &rest[at + 1..];
    let user = credentials.split(':').next().unwrap_or("user");
    format!("{scheme}{user}:***@{host}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_url_redaction_hides_password() {
        assert_eq!(
            redact_database_url("postgres://football_app:secret@localhost:5432/football_model"),
            "postgres://football_app:***@localhost:5432/football_model"
        );
    }
}
