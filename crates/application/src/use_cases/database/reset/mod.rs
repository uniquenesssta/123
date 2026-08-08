use crate::ports::{
    database::{DatabaseHealthSnapshot, DatabaseLifecyclePort, DatabaseObservabilityPort},
    PortError, PortErrorKind, PortResult,
};

pub(crate) async fn validate_confirmation<T>(
    port: &T,
    confirmation: &str,
) -> PortResult<DatabaseHealthSnapshot>
where
    T: DatabaseObservabilityPort + ?Sized,
{
    let health = super::health::execute(port).await?;
    if confirmation.trim() != health.database_name {
        return Err(PortError::new(
            PortErrorKind::InvalidState,
            format!(
                "确认名称不匹配。请输入当前数据库名称：{}",
                health.database_name
            ),
        ));
    }
    Ok(health)
}

pub(crate) async fn execute<T>(
    port: &T,
    expected_database_name: &str,
) -> PortResult<DatabaseHealthSnapshot>
where
    T: DatabaseLifecyclePort + DatabaseObservabilityPort + ?Sized,
{
    let configured_health = super::health::execute(port).await?;
    if configured_health.database_name != expected_database_name {
        return Err(PortError::new(
            PortErrorKind::InvalidState,
            "保存的连接配置与当前数据库不一致，已拒绝清空",
        ));
    }

    if let Err(error) = port.reset_to_pristine().await {
        let _ = super::migrate::execute(port).await;
        return Err(PortError::new(
            error.kind,
            format!("彻底清空数据库失败：{}", error.message),
        ));
    }

    super::health::execute(port).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::database::DatabaseStatistics;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeDatabase {
        database_name: String,
        resets: AtomicUsize,
    }

    impl FakeDatabase {
        fn new(database_name: &str) -> Self {
            Self {
                database_name: database_name.to_string(),
                resets: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl DatabaseLifecyclePort for FakeDatabase {
        async fn migrate(&self) -> PortResult<()> {
            Ok(())
        }

        async fn recover_interrupted_work(&self) -> PortResult<()> {
            Ok(())
        }

        async fn reset_to_pristine(&self) -> PortResult<()> {
            self.resets.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn close(&self) -> PortResult<()> {
            Ok(())
        }
    }

    #[async_trait]
    impl DatabaseObservabilityPort for FakeDatabase {
        async fn health(&self) -> PortResult<DatabaseHealthSnapshot> {
            Ok(DatabaseHealthSnapshot {
                connected: true,
                database_name: self.database_name.clone(),
                server_version: "test".to_string(),
                migration_count: 46,
                database_size_bytes: 0,
                checked_at: Utc::now(),
                latency_ms: 0,
            })
        }

        async fn statistics(&self) -> PortResult<DatabaseStatistics> {
            Ok(DatabaseStatistics {
                competitions: 0,
                teams: 0,
                players: 0,
                matches: 0,
                model_runs: 0,
                rule_packages: 0,
                route_bindings: 0,
                ability_observations: 0,
                pending_ability_updates: 0,
                data_providers: 0,
                availability_records: 0,
                active_lineups: 0,
                large_counts_are_estimates: false,
            })
        }
    }

    #[tokio::test]
    async fn confirmation_must_match_database_name() {
        let database = FakeDatabase::new("football_test");
        let error = validate_confirmation(&database, "other")
            .await
            .expect_err("mismatched confirmation must fail");
        assert_eq!(error.kind, PortErrorKind::InvalidState);
        assert_eq!(
            error.message,
            "确认名称不匹配。请输入当前数据库名称：football_test"
        );
        assert_eq!(database.resets.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn reset_requires_the_same_database_before_destructive_work() {
        let database = FakeDatabase::new("football_test");
        let health = execute(&database, "football_test")
            .await
            .expect("reset use case");
        assert_eq!(health.database_name, "football_test");
        assert_eq!(database.resets.load(Ordering::SeqCst), 1);
    }
}
