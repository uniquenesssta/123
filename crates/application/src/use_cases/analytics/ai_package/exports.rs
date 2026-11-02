use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_analysis_package::{response_template_bytes, write_analysis_package};
use football_domain::AiAnalysisPackageSummary;
use std::{fs, path::Path};
use uuid::Uuid;

pub(crate) fn export_ai_analysis_response_template(
    output_path: String,
    source_package_id: Option<Uuid>,
) -> ApplicationResult<String> {
    let bytes = response_template_bytes(source_package_id)?;
    fs::write(&output_path, bytes)?;
    Ok(output_path)
}

pub(crate) async fn export_ai_analysis_package<P>(
    port: &P,
    output_path: String,
    expected_snapshot_id: Option<Uuid>,
) -> ApplicationResult<AiAnalysisPackageSummary>
where
    P: AnalyticsPort + ?Sized,
{
    let data = port.build_ai_analysis_data().await?;
    if let Some(expected) = expected_snapshot_id {
        if data
            .overview
            .source_snapshot
            .as_ref()
            .map(|source| source.id)
            != Some(expected)
        {
            return Err(crate::ApplicationError::Validation(
                "分析快照已变化，请刷新后重新确认导出".to_string(),
            ));
        }
    }
    let summary = write_analysis_package(Path::new(&output_path), &data)?;
    port.record_ai_export(&summary).await?;
    Ok(summary)
}
