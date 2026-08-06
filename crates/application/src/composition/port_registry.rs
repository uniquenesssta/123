pub(crate) use football_persistence_postgres::{
    DatabaseHealth, DatabaseOptions, DatabaseStats, ModelRunListItem, PersistenceError,
    PostgresStore as PersistenceStore,
};
use tokio::sync::RwLock;

pub(crate) struct ActiveDatabase {
    pub(crate) store: PersistenceStore,
    pub(crate) redacted_url: String,
}

pub(crate) struct PortRegistry {
    database: RwLock<Option<ActiveDatabase>>,
}

impl PortRegistry {
    pub(crate) fn new() -> Self {
        Self {
            database: RwLock::new(None),
        }
    }

    pub(crate) fn into_database(self) -> RwLock<Option<ActiveDatabase>> {
        self.database
    }
}
