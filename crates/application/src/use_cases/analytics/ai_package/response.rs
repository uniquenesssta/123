use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_analysis_package::read_analysis_response;
use football_domain::{AiAnalysisResponsePreview, AiAnalysisSuggestionRecord};
use std::path::Path;

pub(crate) fn preview_ai_analysis_response(
    input_path: String,
) -> ApplicationResult<AiAnalysisResponsePreview> {
    Ok(read_analysis_response(Path::new(&input_path))?)
}

pub(crate) async fn import_ai_analysis_response<P>(
    port: &P,
    input_path: String,
) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>>
where
    P: AnalyticsPort + ?Sized,
{
    let preview = read_analysis_response(Path::new(&input_path))?;
    Ok(port.import_ai_response(&input_path, &preview).await?)
}
