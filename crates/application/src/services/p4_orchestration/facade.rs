use crate::{ApplicationError, ApplicationResult, ApplicationService};
use serde_json::Value;
use std::sync::Arc;

impl ApplicationService {
    pub async fn process_next_p4_orchestration_job(
        self: &Arc<Self>,
    ) -> ApplicationResult<Option<Value>> {
        let session = self
            .database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)?;
        self.p4_orchestration
            .process_next(&session, &self.registry, &self.research, &self.prediction)
            .await
    }

    pub fn ensure_p4_orchestration_worker(self: &Arc<Self>) {
        self.p4_orchestration.start(Arc::clone(self));
    }
}
