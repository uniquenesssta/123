use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_domain::{AiAnalysisSuggestionRecord, AiSuggestionDecisionDraft};

pub(crate) async fn list_ai_analysis_suggestions<P>(
    port: &P,
    status: Option<String>,
    limit: u32,
) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>>
where
    P: AnalyticsPort + ?Sized,
{
    Ok(port.list_ai_suggestions(status.as_deref(), limit).await?)
}

pub(crate) async fn decide_ai_analysis_suggestion<P>(
    port: &P,
    draft: AiSuggestionDecisionDraft,
) -> ApplicationResult<AiAnalysisSuggestionRecord>
where
    P: AnalyticsPort + ?Sized,
{
    Ok(port.decide_ai_suggestion(&draft).await?)
}
