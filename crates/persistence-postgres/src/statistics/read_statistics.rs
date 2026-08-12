use super::DatabaseStats;
use crate::{PersistenceResult, PostgresStore};
use sqlx::Row;

impl PostgresStore {
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
