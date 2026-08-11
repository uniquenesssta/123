use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_domain::AiAnalysisSuggestionRecord;

pub(crate) async fn execute<P>(
    port: &P,
    status: Option<String>,
    limit: u32,
) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>>
where
    P: AnalyticsPort + ?Sized,
{
    Ok(port.list_ai_suggestions(status.as_deref(), limit).await?)
}
