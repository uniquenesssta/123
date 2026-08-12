use crate::model_registry::ModelRegistry;
use crate::services::{prediction::PredictionService, research::ResearchService};
use crate::use_cases::p4_orchestration::{process_next, P4OrchestrationAccess};
use crate::ApplicationResult;
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub(crate) struct P4OrchestrationService {
    running: AtomicBool,
}

impl P4OrchestrationService {
    pub(crate) fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
        }
    }

    pub(crate) async fn process_next<P: P4OrchestrationAccess>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        research: &ResearchService,
        prediction: &PredictionService,
    ) -> ApplicationResult<Option<Value>> {
        process_next::execute(port, registry, research, prediction).await
    }

    pub(crate) fn start(&self, application: Arc<crate::ApplicationService>) {
        super::worker::spawn(application);
    }

    pub(super) fn try_mark_running(&self) -> bool {
        !self.running.swap(true, Ordering::SeqCst)
    }

    pub(super) fn mark_stopped(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
