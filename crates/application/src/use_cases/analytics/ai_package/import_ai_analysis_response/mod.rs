use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_analysis_package::read_analysis_response;
use football_domain::AiAnalysisSuggestionRecord;
use std::path::Path;

pub(crate) async fn execute<P>(
    port: &P,
    input_path: String,
) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>>
where
    P: AnalyticsPort + ?Sized,
{
    let preview = read_analysis_response(Path::new(&input_path))?;
    Ok(port.import_ai_response(&input_path, &preview).await?)
}
