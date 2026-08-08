use crate::composition::ApplicationComposition;
use crate::model_registry::ModelRegistry;
use crate::services::database::DatabaseService;
use std::sync::atomic::AtomicBool;

pub struct ApplicationService {
    pub(crate) registry: ModelRegistry,
    pub(crate) database: DatabaseService,
    pub(crate) p4_worker_running: AtomicBool,
}

impl ApplicationService {
    pub fn new() -> Self {
        let (registry, database, p4_worker_running) = ApplicationComposition::new().into_parts();
        Self {
            registry,
            database,
            p4_worker_running,
        }
    }
}

impl Default for ApplicationService {
    fn default() -> Self {
        Self::new()
    }
}
