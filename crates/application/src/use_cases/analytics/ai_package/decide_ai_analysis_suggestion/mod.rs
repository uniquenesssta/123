use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_domain::{AiAnalysisSuggestionRecord, AiSuggestionDecisionDraft};

pub(crate) async fn execute<P>(
    port: &P,
    draft: AiSuggestionDecisionDraft,
) -> ApplicationResult<AiAnalysisSuggestionRecord>
where
    P: AnalyticsPort + ?Sized,
{
    Ok(port.decide_ai_suggestion(&draft).await?)
}
