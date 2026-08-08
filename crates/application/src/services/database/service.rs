use crate::composition::{ActiveDatabase, DatabaseOptions, PersistenceStore, PortRegistry};
use crate::ports::{
    database::{DatabaseHealthSnapshot, DatabaseLifecyclePort},
    PortResult,
};
use crate::use_cases::database::{connect, health, reset};
use tokio::sync::RwLock;

pub(crate) struct PreparedDatabaseConnection {
    session: ActiveDatabase,
}

impl PreparedDatabaseConnection {
    pub(crate) fn transition_store(&self) -> PersistenceStore {
        self.session.transition_store()
    }

    pub(crate) async fn health(&self) -> PortResult<DatabaseHealthSnapshot> {
        health::execute(&self.session).await
    }

    pub(crate) async fn close(&self) {
        let _ = self.session.close().await;
    }
}

pub(crate) struct DatabaseService {
    ports: PortRegistry,
    pub(crate) session: RwLock<Option<ActiveDatabase>>,
}

impl DatabaseService {
    pub(crate) fn new(ports: PortRegistry) -> Self {
        Self {
            ports,
            session: RwLock::new(None),
        }
    }

    pub(crate) async fn prepare_connection(
        &self,
        options: &DatabaseOptions,
    ) -> PortResult<PreparedDatabaseConnection> {
        let session = self.ports.connect_database(options).await?;
        if let Err(error) = connect::execute(&session).await {
            let _ = session.close().await;
            return Err(error);
        }
        Ok(PreparedDatabaseConnection { session })
    }

    pub(crate) async fn activate(&self, prepared: PreparedDatabaseConnection) -> PortResult<()> {
        let previous = self.session.write().await.replace(prepared.session);
        if let Some(previous) = previous {
            previous.close().await?;
        }
        Ok(())
    }

    pub(crate) async fn is_connected(&self) -> bool {
        self.session.read().await.is_some()
    }

    pub(crate) async fn disconnect(&self) -> PortResult<()> {
        let active = self.session.write().await.take();
        if let Some(active) = active {
            active.close().await?;
        }
        Ok(())
    }

    pub(crate) async fn active_session(&self) -> Option<ActiveDatabase> {
        self.session.read().await.clone()
    }

    pub(crate) async fn active_store(&self) -> Option<PersistenceStore> {
        self.session
            .read()
            .await
            .as_ref()
            .map(ActiveDatabase::transition_store)
    }

    pub(crate) async fn preflight_reset(
        &self,
        options: &DatabaseOptions,
        confirmation: &str,
    ) -> PortResult<DatabaseHealthSnapshot> {
        let verification = self.ports.connect_database(options).await?;
        let result = reset::validate_confirmation(&verification, confirmation).await;
        let _ = verification.close().await;
        result
    }

    pub(crate) async fn reset_to_pristine(
        &self,
        options: &DatabaseOptions,
        confirmation: &str,
    ) -> PortResult<()> {
        let current_health = self.preflight_reset(options, confirmation).await?;
        self.disconnect().await?;

        let reset_session = self.ports.connect_database(options).await?;
        let result = reset::execute(&reset_session, &current_health.database_name).await;
        let _ = reset_session.close().await;
        result.map(|_| ())
    }
}
