mod context;
mod operations;
mod sessions;

use crate::{ApplicationError, ApplicationResult, ApplicationService};

impl ApplicationService {
    async fn ai_workspace_session(&self) -> ApplicationResult<crate::composition::ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }
}
