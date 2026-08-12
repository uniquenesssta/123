use crate::{
    ports::exchange::SpreadsheetExchangePort,
    use_cases::exchange::file_validation::spreadsheet::validate_existing_xlsx_path,
    ApplicationError, ApplicationResult,
};
use football_domain::{SpreadsheetImportMode, SpreadsheetImportPreview};
use football_spreadsheet_io::{
    read_player_catalog_workbook, read_player_monthly_workbook, read_team_monthly_workbook,
    read_team_package_workbook,
};
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    input_path: String,
    mode: SpreadsheetImportMode,
) -> ApplicationResult<SpreadsheetImportPreview>
where
    P: SpreadsheetExchangePort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = validate_existing_xlsx_path(&input_path)?;
    let parsed = tokio::task::spawn_blocking(move || {
        match read_player_monthly_workbook(&path).or_else(|_| read_player_catalog_workbook(&path)) {
            Ok(parsed) => Ok(parsed),
            Err(_original) if read_team_monthly_workbook(&path).is_ok() => Err(
                "检测到 football.team-monthly.v1 球队月度工作包。请使用“Excel 工作包 → 球队月度”导入；球员入口不会把球队文件误写为球员。".to_string(),
            ),
            Err(_original) if read_team_package_workbook(&path).is_ok() => Err(
                "检测到 football.team-package.v1 球队完整资料包。请使用“球队 → 导入资料包”统一预检球队与球员链路。".to_string(),
            ),
            Err(original) => Err(format!("球员工作包读取失败：{original}")),
        }
    })
    .await
    .map_err(|error| ApplicationError::Validation(format!("球员 Excel 读取任务失败：{error}")))?
    .map_err(ApplicationError::Validation)?;
    let port = session.await?;
    Ok(port.preview_import(&parsed, mode).await?)
}
