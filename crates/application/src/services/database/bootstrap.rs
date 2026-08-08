use crate::use_cases::database::{health, statistics};
use crate::{
    composition::{database_health_from_snapshot, database_stats_from_statistics},
    default_match, default_rule_package_template, ApplicationResult, ApplicationService,
    BootstrapData,
};

impl ApplicationService {
    pub async fn bootstrap(&self) -> ApplicationResult<BootstrapData> {
        let active_database = self.database.active_session().await;
        let database_configured = active_database.is_some();
        let (
            database_url,
            database_health,
            stats,
            competitions,
            seasons,
            stages,
            rounds,
            rule_packages,
            competition_bindings,
            recent_runs,
        ) = if let Some(active) = active_database {
            let store = active.transition_store();
            (
                Some(active.redacted_url().to_string()),
                Some(database_health_from_snapshot(
                    health::execute(&active).await?,
                )),
                Some(database_stats_from_statistics(
                    statistics::execute(&active).await?,
                )),
                store.list_competitions().await?,
                store.list_seasons().await?,
                store.list_stages().await?,
                store.list_rounds().await?,
                store.list_rule_packages().await?,
                store.list_competition_bindings().await?,
                store.list_recent_runs(50).await?,
            )
        } else {
            (
                None,
                None,
                None,
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            )
        };

        Ok(BootstrapData {
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            database_configured,
            database_url,
            database_health,
            stats,
            models: self.registry.descriptors(),
            competitions,
            seasons,
            stages,
            rounds,
            rule_packages,
            competition_bindings,
            recent_runs,
            default_match: default_match(),
            default_rule_package: default_rule_package_template(),
        })
    }
}
