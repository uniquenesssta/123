use crate::{
    ports::exchange::SpreadsheetExchangePort,
    use_cases::exchange::file_validation::spreadsheet::validate_xlsx_path, ApplicationError,
    ApplicationResult,
};
use football_domain::SpreadsheetExportSummary;
use football_spreadsheet_io::write_player_monthly_template;
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    output_path: String,
) -> ApplicationResult<SpreadsheetExportSummary>
where
    P: SpreadsheetExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = validate_xlsx_path(&output_path)?;
    let port = session.await?;
    let references = port.reference_data().await?;
    let output = path.clone();
    tokio::task::spawn_blocking(move || write_player_monthly_template(&output, &references))
        .await
        .map_err(|error| ApplicationError::Validation(format!("模板导出任务失败：{error}")))??;
    Ok(SpreadsheetExportSummary {
        output_path: path.to_string_lossy().to_string(),
        team_count: 0,
        player_count: 0,
        related_row_count: 0,
    })
}
