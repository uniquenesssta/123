use crate::ApplicationResult;
use football_analysis_package::read_analysis_response;
use football_domain::AiAnalysisResponsePreview;
use std::path::Path;

pub(crate) fn execute(input_path: String) -> ApplicationResult<AiAnalysisResponsePreview> {
    Ok(read_analysis_response(Path::new(&input_path))?)
}
