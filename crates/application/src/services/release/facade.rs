use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    ReleaseAcceptanceRequest, ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary,
};
use uuid::Uuid;

impl ApplicationService {
    async fn release_session(&self) -> ApplicationResult<crate::composition::DatabaseSession> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    pub async fn run_release_acceptance(
        &self,
        request: ReleaseAcceptanceRequest,
    ) -> ApplicationResult<ReleaseAcceptanceRun> {
        self.release
            .run_acceptance(self.release_session(), request)
            .await
    }

    pub async fn list_release_acceptance_runs(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<ReleaseAcceptanceRunSummary>> {
        self.release.list_runs(self.release_session(), limit).await
    }

    pub async fn read_release_acceptance_run(
        &self,
        run_id: Uuid,
    ) -> ApplicationResult<ReleaseAcceptanceRun> {
        self.release.read_run(self.release_session(), run_id).await
    }
}
