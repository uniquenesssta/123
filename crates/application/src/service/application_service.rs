use crate::composition::{ActiveDatabase, ApplicationComposition};
use crate::model_registry::ModelRegistry;
use std::sync::atomic::AtomicBool;
use tokio::sync::RwLock;

pub struct ApplicationService {
    pub(crate) registry: ModelRegistry,
    pub(crate) database: RwLock<Option<ActiveDatabase>>,
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
