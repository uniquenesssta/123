use crate::{
    ports::exchange::SpreadsheetExchangePort,
    use_cases::exchange::file_validation::spreadsheet::validate_xlsx_path, ApplicationError,
    ApplicationResult,
};
use football_domain::{MonthlyWorkbookExportSummary, MonthlyWorkbookKind};
use football_spreadsheet_io::write_team_monthly_template;
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    output_path: String,
) -> ApplicationResult<MonthlyWorkbookExportSummary>
where
    P: SpreadsheetExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = validate_xlsx_path(&output_path)?;
    let port = session.await?;
    let references = port.reference_data().await?;
    let output = path.clone();
    tokio::task::spawn_blocking(move || write_team_monthly_template(&output, &references))
        .await
        .map_err(|error| ApplicationError::Validation(format!("球队模板导出失败：{error}")))??;
    Ok(MonthlyWorkbookExportSummary {
        output_path: path.to_string_lossy().to_string(),
        workbook_kind: MonthlyWorkbookKind::Team,
        team_count: 0,
        player_count: 0,
        coach_count: 0,
        related_row_count: 0,
        data_gap_count: 0,
    })
}
