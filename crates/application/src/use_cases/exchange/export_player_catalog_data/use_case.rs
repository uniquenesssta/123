use crate::{
    ports::exchange::SpreadsheetExchangePort,
    use_cases::exchange::file_validation::spreadsheet::validate_xlsx_path, ApplicationError,
    ApplicationResult,
};
use football_domain::SpreadsheetExportSummary;
use football_spreadsheet_io::write_player_monthly_export;
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
    let data = port.export_data().await?;
    let gaps = port.data_gaps().await?;
    let team_count = data.teams.len() as u64;
    let player_count = data.players.len() as u64;
    let related_row_count = (data.names.len()
        + data.positions.len()
        + data.team_periods.len()
        + data.abilities.len()
        + data.availability.len()
        + data.dynamic_tags.len()) as u64;
    let output = path.clone();
    tokio::task::spawn_blocking(move || {
        write_player_monthly_export(&output, &references, &data, &gaps)
    })
    .await
    .map_err(|error| ApplicationError::Validation(format!("数据导出任务失败：{error}")))??;
    Ok(SpreadsheetExportSummary {
        output_path: path.to_string_lossy().to_string(),
        team_count,
        player_count,
        related_row_count,
    })
}
