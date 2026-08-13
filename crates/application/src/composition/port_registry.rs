use football_persistence_postgres::register_adapters;
pub(crate) use football_persistence_postgres::{
    DatabaseHealth, DatabaseOptions, DatabaseStats, ModelRunListItem, PersistenceError,
    PostgresStore as PersistenceStore,
};

use super::adapters::map_persistence_error;
use crate::ports::PortResult;

pub(crate) type DatabaseSession = PersistenceStore;

pub(crate) struct PortRegistry;

impl PortRegistry {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn connect_database(
        &self,
        options: &DatabaseOptions,
    ) -> PortResult<DatabaseSession> {
        register_adapters(options)
            .await
            .map_err(map_persistence_error)
    }
}
