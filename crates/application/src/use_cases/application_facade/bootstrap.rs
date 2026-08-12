use crate::composition::{database_health_from_snapshot, database_stats_from_statistics};
use crate::services::prediction::compatibility as prediction_compatibility;
use crate::use_cases::database::{health, statistics};
use crate::{
    default_match, default_rule_package_template, ApplicationResult, ApplicationService,
    BootstrapData,
};

pub(crate) async fn execute(application: &ApplicationService) -> ApplicationResult<BootstrapData> {
    let active_database = application.database.active_session().await;
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
        let hierarchy = application.competition.load_hierarchy(&active).await?;
        let rules = application.rules.load_catalog(&active).await?;
        (
            Some(active.redacted_url().to_string()),
            Some(database_health_from_snapshot(
                health::execute(&active).await?,
            )),
            Some(database_stats_from_statistics(
                statistics::execute(&active).await?,
            )),
            hierarchy.competitions,
            hierarchy.seasons,
            hierarchy.stages,
            hierarchy.rounds,
            rules.rule_packages,
            rules.competition_bindings,
            prediction_compatibility::list_recent_runs(&application.prediction, &active, 50)
                .await?,
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
        models: application.registry.descriptors(),
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
