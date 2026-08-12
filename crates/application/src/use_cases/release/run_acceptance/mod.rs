mod checks;
mod report;
mod summary;
mod validation;

use crate::{ports::release::ReleaseAcceptancePort, ApplicationResult};
use chrono::Utc;
use football_domain::{
    ReleaseAcceptanceRequest, ReleaseAcceptanceRun, RELEASE_ACCEPTANCE_CONTRACT_VERSION,
    RELEASE_ACCEPTANCE_FIXTURE_VERSION,
};
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    request: ReleaseAcceptanceRequest,
) -> ApplicationResult<ReleaseAcceptanceRun>
where
    P: ReleaseAcceptancePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let request = validation::normalize_request(request)?;
    let started_at = Utc::now();
    let run_id = Uuid::new_v4();
    let port = session.await?;
    let runtime = port
        .runtime_facts(request.performance_window_days, request.cost_window_days)
        .await?;
    let checks = checks::build(run_id, &runtime, &request);
    let summary = summary::build(&checks, &runtime, &request);
    let completed_at = Utc::now();
    let report_sha256 = report::hash(
        run_id,
        started_at,
        completed_at,
        &request,
        &summary,
        &checks,
    )?;
    let run = ReleaseAcceptanceRun {
        id: run_id,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        contract_version: RELEASE_ACCEPTANCE_CONTRACT_VERSION.to_string(),
        fixture_version: RELEASE_ACCEPTANCE_FIXTURE_VERSION.to_string(),
        overall_status: summary.overall_status,
        started_at,
        completed_at,
        requested_by: request.requested_by,
        report_sha256,
        passed_count: summary.passed_count,
        warning_count: summary.warning_count,
        blocked_count: summary.blocked_count,
        category_summaries: summary.category_summaries,
        performance: summary.performance,
        cost: summary.cost,
        checks,
    };
    port.persist_run(&run).await?;
    Ok(run)
}
