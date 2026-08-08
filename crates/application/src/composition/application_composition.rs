use super::PortRegistry;
use crate::model_registry::ModelRegistry;
use crate::model_shell::PublicModelStub;
use crate::services::database::DatabaseService;
use std::sync::{atomic::AtomicBool, Arc};

pub(crate) struct ApplicationComposition {
    registry: ModelRegistry,
    database: DatabaseService,
    p4_worker_running: AtomicBool,
}

impl ApplicationComposition {
    pub(crate) fn new() -> Self {
        let mut registry = ModelRegistry::new();
        for model in PublicModelStub::built_in_models() {
            registry.register(Arc::new(model));
        }
        let database = DatabaseService::new(PortRegistry::new());

        Self {
            registry,
            database,
            p4_worker_running: AtomicBool::new(false),
        }
    }

    pub(crate) fn into_parts(self) -> (ModelRegistry, DatabaseService, AtomicBool) {
        (self.registry, self.database, self.p4_worker_running)
    }
}
