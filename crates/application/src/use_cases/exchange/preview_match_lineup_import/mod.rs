use crate::{ports::exchange::MatchLineupExchangePort, ApplicationError, ApplicationResult};
use football_domain::{SpreadsheetImportMode, SpreadsheetImportPreview};
use football_spreadsheet_io::read_match_lineup_workbook;
use std::future::Future;

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
    let path = file_validation::validate_input(&input_path, "xlsx")?;
    let parsed = tokio::task::spawn_blocking(move || read_match_lineup_workbook(&path))
        .await
        .map_err(|error| {
            ApplicationError::Validation(format!("比赛 Excel 读取任务失败：{error}"))
        })??;
    let port = session.await?;
    Ok(port.preview_import(&parsed, mode).await?)
}
