use crate::{ports::exchange::MatchLineupExchangePort, ApplicationError, ApplicationResult};
use football_domain::{SpreadsheetImportMode, SpreadsheetImportPreview};
use football_spreadsheet_io::{extract_ai_match_workbook, read_match_lineup_workbook};
use std::future::Future;
use tempfile::tempdir;

use super::file_validation;

pub(crate) async fn execute<P, F>(
    session: F,
    input_path: String,
    mode: SpreadsheetImportMode,
) -> ApplicationResult<SpreadsheetImportPreview>
where
    P: MatchLineupExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = file_validation::validate_input(&input_path, "zip")?;
    let temp = tempdir().map_err(|error| ApplicationError::Validation(error.to_string()))?;
    let workbook_path = temp.path().join("match_and_lineup.xlsx");
    let parsed = tokio::task::spawn_blocking(move || {
        extract_ai_match_workbook(&path, &workbook_path)?;
        read_match_lineup_workbook(&workbook_path)
    })
    .await
    .map_err(|error| ApplicationError::Validation(format!("AI 分析包读取任务失败：{error}")))??;
    let port = session.await?;
    Ok(port.preview_import(&parsed, mode).await?)
}
