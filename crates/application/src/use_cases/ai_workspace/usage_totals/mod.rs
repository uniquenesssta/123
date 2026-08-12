use crate::{
    ports::{ai_workspace::ApiWorkspaceSessionPort, research::ResearchGatewayAuditPort},
    ApplicationResult,
};
use football_domain::OpenAiUsageTotals;
use std::future::Future;

pub(crate) async fn execute<P, F>(session: F) -> ApplicationResult<OpenAiUsageTotals>
where
    P: ApiWorkspaceSessionPort + ResearchGatewayAuditPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let port = session.await?;
    let formal = ResearchGatewayAuditPort::usage_totals(&port).await?;
    let workspace = ApiWorkspaceSessionPort::usage_totals(&port).await?;
    Ok(OpenAiUsageTotals {
        today_cost_usd: formal.today_cost_usd + workspace.today_cost_usd,
        month_cost_usd: formal.month_cost_usd + workspace.month_cost_usd,
        today_request_count: formal
            .today_request_count
            .saturating_add(workspace.today_request_count),
        month_request_count: formal
            .month_request_count
            .saturating_add(workspace.month_request_count),
    })
}
