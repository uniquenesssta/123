use super::{
    default_match, default_rule_package_template, rule_packages::built_in_rule_packages,
    ActiveDatabase, ApplicationError, ApplicationResult, ApplicationService, BootstrapData,
    DatabaseHealth, DatabaseOptions, PersistenceStore,
};
use std::sync::Arc;

impl ApplicationService {
    pub async fn connect_database(
        self: &Arc<Self>,
        options: DatabaseOptions,
    ) -> ApplicationResult<DatabaseHealth> {
        let store = PersistenceStore::connect(&options).await?;
        store.migrate().await?;
        store.recover_interrupted_jobs().await?;
        store.recover_interrupted_api_workspace_operations().await?;
        self.register_built_in_rule_packages(&store).await?;
        self.register_p4_persistence_artifacts(&store).await?;
        self.register_openai_research_artifacts(&store).await?;
        let health = store.health().await?;
        let active = ActiveDatabase {
            store: store.clone(),
            redacted_url: options.redacted_url(),
        };
        let previous = self.database.write().await.replace(active);
        if let Some(previous) = previous {
            previous.store.close().await;
        }
        crate::analytics::spawn_job_worker(store);
        self.ensure_p4_orchestration_worker();
        Ok(health)
    }

    pub fn ensure_p4_orchestration_worker(self: &Arc<Self>) {
        crate::p4_orchestration::spawn_p4_orchestration_worker(Arc::clone(self));
    }

    pub async fn is_database_connected(&self) -> bool {
        self.database.read().await.is_some()
    }

    pub async fn disconnect_database(&self) {
        let active = {
            let mut database = self.database.write().await;
            database.take()
        };
        if let Some(active) = active {
            active.store.close().await;
        }
    }

    pub async fn bootstrap(&self) -> ApplicationResult<BootstrapData> {
        let active_database = {
            let database = self.database.read().await;
            database
                .as_ref()
                .map(|active| (active.redacted_url.clone(), active.store.clone()))
        };
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
        ) = if let Some((redacted_url, store)) = active_database {
            (
                Some(redacted_url),
                Some(store.health().await?),
                Some(store.stats().await?),
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
    pub(super) async fn active_store(&self) -> ApplicationResult<PersistenceStore> {
        self.database
            .read()
            .await
            .as_ref()
            .map(|active| active.store.clone())
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    async fn register_built_in_rule_packages(
        &self,
        store: &PersistenceStore,
    ) -> ApplicationResult<()> {
        for draft in built_in_rule_packages() {
            let model = self
                .registry
                .get(&draft.routing.model_id)
                .ok_or_else(|| ApplicationError::ModelNotFound(draft.routing.model_id.clone()))?;
            model
                .validate_parameters(&draft.parameters)
                .map_err(|error| ApplicationError::Model(error.to_string()))?;
            let summary = store
                .register_rule_package(&model.descriptor(), &draft)
                .await?;
            if draft.routing.activate_as_type_default {
                store
                    .ensure_type_default_binding(
                        summary.id,
                        draft.competition_profile.competition_kind,
                        draft.routing.priority,
                        &format!("内置默认 · {}", draft.display_name),
                    )
                    .await?;
            }
        }
        Ok(())
    }
}
