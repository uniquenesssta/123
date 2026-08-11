use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_analysis_package::write_analysis_package;
use football_domain::AiAnalysisPackageSummary;
use std::path::Path;

pub(crate) async fn execute<P>(
    port: &P,
    output_path: String,
) -> ApplicationResult<AiAnalysisPackageSummary>
where
    P: AnalyticsPort + ?Sized,
{
    let data = port.build_ai_analysis_data().await?;
    let summary = write_analysis_package(Path::new(&output_path), &data)?;
    port.record_ai_export(&summary).await?;
    Ok(summary)
}
