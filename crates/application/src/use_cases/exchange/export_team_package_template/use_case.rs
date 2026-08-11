use crate::{
    ports::exchange::SpreadsheetExchangePort,
    use_cases::exchange::file_validation::spreadsheet::validate_xlsx_path,
    ApplicationError, ApplicationResult,
};
use football_domain::{TeamPackageExportSummary, TEAM_PACKAGE_FORMAT};
use football_spreadsheet_io::write_team_package_template;
use std::future::Future;

pub(crate) async fn execute<P, F>(session: F, output_path: String) -> ApplicationResult<TeamPackageExportSummary>
where
    P: SpreadsheetExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = validate_xlsx_path(&output_path)?;
    let port = session.await?;
    let references = port.reference_data().await?;
    let output = path.clone();
    tokio::task::spawn_blocking(move || write_team_package_template(&output, &references))
        .await
        .map_err(|error| ApplicationError::Validation(format!("球队完整资料包模板导出失败：{error}")))??;
    Ok(TeamPackageExportSummary {
        output_path: path.to_string_lossy().to_string(),
        format_version: TEAM_PACKAGE_FORMAT.to_string(),
        visible_sheet_count: 7,
    })
}
