use super::service::PreparedDatabaseConnection;
use crate::{
    composition::{
        database_health_from_snapshot, DatabaseHealth, DatabaseOptions, PersistenceStore,
    },
    rule_packages::built_in_rule_packages,
    ApplicationError, ApplicationResult, ApplicationService,
};
use std::sync::Arc;

impl ApplicationService {
    pub async fn connect_database(
        self: &Arc<Self>,
        options: DatabaseOptions,
    ) -> ApplicationResult<DatabaseHealth> {
        let prepared = self.database.prepare_connection(&options).await?;
        if let Err(error) = self.initialize_database_contents(&prepared).await {
            prepared.close().await;
            return Err(error);
        }

        let health = match prepared.health().await {
            Ok(health) => database_health_from_snapshot(health),
            Err(error) => {
                prepared.close().await;
                return Err(error.into());
            }
        };
        self.database.activate(prepared).await?;

        let store = self.active_store().await?;
        crate::analytics::spawn_job_worker(store);
        self.ensure_p4_orchestration_worker();
        Ok(health)
    }

    pub fn ensure_p4_orchestration_worker(self: &Arc<Self>) {
        crate::p4_orchestration::spawn_p4_orchestration_worker(Arc::clone(self));
    }

    pub async fn is_database_connected(&self) -> bool {
        self.database.is_connected().await
    }

    pub async fn disconnect_database(&self) {
        let _ = self.database.disconnect().await;
    }

    pub async fn preflight_database_reset(
        &self,
        options: &DatabaseOptions,
        confirmation: &str,
    ) -> ApplicationResult<DatabaseHealth> {
        let health = self.database.preflight_reset(options, confirmation).await?;
        Ok(database_health_from_snapshot(health))
    }

    pub async fn reset_database(
        self: &Arc<Self>,
        options: DatabaseOptions,
        confirmation: String,
    ) -> ApplicationResult<DatabaseHealth> {
        if let Err(error) = self
            .database
            .reset_to_pristine(&options, &confirmation)
            .await
        {
            if !self.database.is_connected().await {
                let _ = self.connect_database(options.clone()).await;
            }
            return Err(error.into());
        }
        self.connect_database(options).await
    }

    pub(crate) async fn active_store(&self) -> ApplicationResult<PersistenceStore> {
        self.database
            .active_store()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    async fn initialize_database_contents(
        &self,
        prepared: &PreparedDatabaseConnection,
    ) -> ApplicationResult<()> {
        let store = prepared.transition_store();
        self.register_built_in_rule_packages(&store).await?;
        self.register_p4_persistence_artifacts(&store).await?;
        self.register_openai_research_artifacts(&store).await
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
