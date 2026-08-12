use crate::{
    composition::{database_health_from_snapshot, DatabaseHealth, DatabaseOptions},
    use_cases::application_facade::database_lifecycle,
    ApplicationResult, ApplicationService,
};
use std::sync::Arc;

impl ApplicationService {
    pub async fn connect_database(
        self: &Arc<Self>,
        options: DatabaseOptions,
    ) -> ApplicationResult<DatabaseHealth> {
        database_lifecycle::connect::execute(self, options).await
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
        database_lifecycle::reset::execute(self, options, confirmation).await
    }
}
