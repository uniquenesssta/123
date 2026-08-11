use crate::{ports::exchange::MatchLineupExchangePort, ApplicationError, ApplicationResult};
use football_domain::AiMatchPackageSummary;
use football_spreadsheet_io::{write_ai_match_package, write_match_lineup_export};
use std::future::Future;
use tempfile::tempdir;
use uuid::Uuid;

use super::file_validation;

pub(crate) async fn execute<P, F>(
    session: F,
    output_path: String,
    match_id: Uuid,
) -> ApplicationResult<AiMatchPackageSummary>
where
    P: MatchLineupExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = file_validation::validate_output(&output_path, "zip")?;
    let port = session.await?;
    let data = port.export_match_lineup(Some(match_id)).await?;
    let context = port.ai_match_package_context(match_id).await?;
    let temp = tempdir().map_err(|error| ApplicationError::Validation(error.to_string()))?;
    let workbook_path = temp.path().join("match_and_lineup.xlsx");
    let output = path.clone();
    tokio::task::spawn_blocking(move || {
        write_match_lineup_export(&workbook_path, &data)?;
        write_ai_match_package(&output, &workbook_path, &context)
    })
    .await
    .map_err(|error| ApplicationError::Validation(format!("AI 分析包导出任务失败：{error}")))?
    .map_err(ApplicationError::Spreadsheet)
}
