use crate::{
    ports::exchange::{MonthlyWorkbookPort, SpreadsheetExchangePort},
    use_cases::exchange::file_validation::spreadsheet::validate_xlsx_path,
    ApplicationError, ApplicationResult,
};
use football_domain::{MonthlyWorkbookExportSummary, MonthlyWorkbookKind};
use football_spreadsheet_io::write_team_monthly_export;
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    output_path: String,
) -> ApplicationResult<MonthlyWorkbookExportSummary>
where
    P: SpreadsheetExchangePort + MonthlyWorkbookPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = validate_xlsx_path(&output_path)?;
    let port = session.await?;
    let references = port.reference_data().await?;
    let data = MonthlyWorkbookPort::export_data(&port).await?;
    let summary = MonthlyWorkbookExportSummary {
        output_path: path.to_string_lossy().to_string(),
        workbook_kind: MonthlyWorkbookKind::Team,
        team_count: data.teams.len() as u64,
        player_count: 0,
        coach_count: data.coaches.len() as u64,
        related_row_count: (data.names.len()
            + data.coach_periods.len()
            + data.formation_usage.len()
            + data.tactical_observations.len()
            + data.ability_observations.len()) as u64,
        data_gap_count: data.data_gaps.len() as u64,
    };
    let output = path.clone();
    tokio::task::spawn_blocking(move || write_team_monthly_export(&output, &references, &data))
        .await
        .map_err(|error| ApplicationError::Validation(format!("球队数据导出失败：{error}")))??;
    Ok(summary)
}
