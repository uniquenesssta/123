use crate::{
    ports::exchange::MonthlyWorkbookPort,
    use_cases::exchange::file_validation::spreadsheet::validate_existing_xlsx_path,
    ApplicationError, ApplicationResult,
};
use football_domain::{SpreadsheetImportMode, SpreadsheetImportPreview};
use football_spreadsheet_io::read_team_monthly_workbook;
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    input_path: String,
    mode: SpreadsheetImportMode,
) -> ApplicationResult<SpreadsheetImportPreview>
where
    P: MonthlyWorkbookPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = validate_existing_xlsx_path(&input_path)?;
    let parsed = tokio::task::spawn_blocking(move || read_team_monthly_workbook(&path))
        .await
        .map_err(|error| ApplicationError::Validation(format!("球队 Excel 读取失败：{error}")))??;
    let port = session.await?;
    Ok(port.preview_import(&parsed, mode).await?)
}
