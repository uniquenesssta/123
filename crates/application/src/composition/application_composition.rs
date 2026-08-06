use super::{ActiveDatabase, PortRegistry};
use crate::model_registry::ModelRegistry;
use crate::model_shell::PublicModelStub;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::RwLock;

pub(crate) struct ApplicationComposition {
    registry: ModelRegistry,
    ports: PortRegistry,
    p4_worker_running: AtomicBool,
}

impl ApplicationComposition {
    pub(crate) fn new() -> Self {
        let mut registry = ModelRegistry::new();
        for model in PublicModelStub::built_in_models() {
            registry.register(Arc::new(model));
        }

        Self {
            registry,
            ports: PortRegistry::new(),
            p4_worker_running: AtomicBool::new(false),
        }
    }

    pub(crate) fn into_parts(self) -> (ModelRegistry, RwLock<Option<ActiveDatabase>>, AtomicBool) {
        (
            self.registry,
            self.ports.into_database(),
            self.p4_worker_running,
        )
    }
}
