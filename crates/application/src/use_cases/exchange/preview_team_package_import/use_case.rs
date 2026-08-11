use super::{coverage, package_rows};
use crate::{
    ports::exchange::{MonthlyWorkbookPort, SpreadsheetExchangePort},
    use_cases::exchange::file_validation::spreadsheet::validate_existing_xlsx_path,
    ApplicationError, ApplicationResult,
};
use football_domain::{
    SpreadsheetImportMode, SpreadsheetParsedWorkbook, TeamPackageImportPreview,
    PLAYER_MONTHLY_FORMAT, TEAM_MONTHLY_FORMAT,
};
use football_spreadsheet_io::{
    read_player_catalog_workbook, read_player_monthly_workbook, read_team_monthly_workbook,
    read_team_package_workbook,
};
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    input_path: String,
    mode: SpreadsheetImportMode,
) -> ApplicationResult<TeamPackageImportPreview>
where
    P: SpreadsheetExchangePort + MonthlyWorkbookPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let path = validate_existing_xlsx_path(&input_path)?;
    let parsed = tokio::task::spawn_blocking(move || match read_team_package_workbook(&path) {
        Ok(parsed) => Ok(parsed),
        Err(_original) if read_team_monthly_workbook(&path).is_ok() => Err(
            "检测到 football.team-monthly.v1 球队月度工作包。请使用“Excel 工作包 → 球队月度”导入；完整资料包入口仅接受 football.team-package.v1。".to_string(),
        ),
        Err(_original)
            if read_player_monthly_workbook(&path).is_ok()
                || read_player_catalog_workbook(&path).is_ok() =>
        {
            Err("检测到球员工作包。请切换到“球队与人员 → 球员 → 球员工作包”导入；当前入口仅接受球队完整资料包。".to_string())
        }
        Err(original) => Err(format!("球队完整资料包读取失败：{original}")),
    })
    .await
    .map_err(|error| ApplicationError::Validation(format!("球队完整资料包读取任务失败：{error}")))?
    .map_err(ApplicationError::Validation)?;

    let package_team_references = package_rows::collect_team_references(&parsed)?;
    let port = session.await?;
    let team_parsed = SpreadsheetParsedWorkbook {
        format_version: TEAM_MONTHLY_FORMAT.to_string(),
        source_file_name: parsed.source_file_name.clone(),
        source_sha256: parsed.source_sha256.clone(),
        rows: package_rows::team_rows(&parsed),
    };
    let player_parsed = SpreadsheetParsedWorkbook {
        format_version: PLAYER_MONTHLY_FORMAT.to_string(),
        source_file_name: parsed.source_file_name.clone(),
        source_sha256: parsed.source_sha256.clone(),
        rows: package_rows::player_rows(&parsed),
    };
    let team_preview = if team_parsed.rows.is_empty() {
        None
    } else {
        Some(MonthlyWorkbookPort::preview_import(&port, &team_parsed, mode).await?)
    };
    let player_preview = if player_parsed.rows.is_empty() {
        None
    } else {
        Some(
            SpreadsheetExchangePort::preview_import_with_team_references(
                &port,
                &player_parsed,
                mode,
                &package_team_references,
            )
            .await?,
        )
    };
    let coverage = coverage::calculate(&parsed, team_preview.as_ref(), player_preview.as_ref());
    Ok(TeamPackageImportPreview {
        source_file_name: parsed.source_file_name,
        source_sha256: parsed.source_sha256,
        team_preview,
        player_preview,
        coverage,
    })
}
