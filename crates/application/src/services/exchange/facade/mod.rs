mod match_lineup;
mod spreadsheet;

use crate::{ApplicationError, ApplicationResult, ApplicationService};

impl ApplicationService {
    async fn exchange_session(&self) -> ApplicationResult<crate::composition::DatabaseSession> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }
}
