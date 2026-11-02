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
    expected: Option<AiAnalysisResponsePreview>,
) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>>
where
    P: AnalyticsPort + ?Sized,
{
    let preview = read_analysis_response(Path::new(&input_path))?;
    if let Some(expected) = expected {
        if serde_json::to_value(&expected)? != serde_json::to_value(&preview)? {
            return Err(crate::ApplicationError::Validation(
                "回包内容或来源已变化，请重新预检并确认".to_string(),
            ));
        }
    }
    Ok(port.import_ai_response(&input_path, &preview).await?)
}
