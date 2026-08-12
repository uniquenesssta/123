use crate::{ports::release::ReleaseAcceptancePort, use_cases::release, ApplicationResult};
use football_domain::{
    ReleaseAcceptanceRequest, ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary,
};
use std::future::Future;
use uuid::Uuid;

#[derive(Debug, Default)]
pub(crate) struct ReleaseService;

impl ReleaseService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn run_acceptance<P, F>(
        &self,
        session: F,
        request: ReleaseAcceptanceRequest,
    ) -> ApplicationResult<ReleaseAcceptanceRun>
    where
        P: ReleaseAcceptancePort,
        F: Future<Output = ApplicationResult<P>>,
    {
        release::run_acceptance::execute(session, request).await
    }

    pub(crate) async fn list_runs<P, F>(
        &self,
        session: F,
        limit: u32,
    ) -> ApplicationResult<Vec<ReleaseAcceptanceRunSummary>>
    where
        P: ReleaseAcceptancePort,
        F: Future<Output = ApplicationResult<P>>,
    {
        release::list_runs::execute(session, limit).await
    }

    pub(crate) async fn read_run<P, F>(
        &self,
        session: F,
        run_id: Uuid,
    ) -> ApplicationResult<ReleaseAcceptanceRun>
    where
        P: ReleaseAcceptancePort,
        F: Future<Output = ApplicationResult<P>>,
    {
        release::read_run::execute(session, run_id).await
    }
}
