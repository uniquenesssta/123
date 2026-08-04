use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    AiMatchPackageSummary, MatchLineupExportSummary, SpreadsheetImportCommitResult,
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
};
use football_spreadsheet_io::{
    extract_ai_match_workbook, read_match_lineup_workbook, write_ai_match_package,
    write_match_lineup_export, write_match_lineup_template,
};
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use uuid::Uuid;

impl ApplicationService {
    pub async fn export_match_lineup_template(
        &self,
        output_path: String,
    ) -> ApplicationResult<MatchLineupExportSummary> {
        let path = validate_output(&output_path, "xlsx")?;
        let store = self.active_store().await?;
        let data = store.match_lineup_export_data(None).await?;
        let player_count = data.players.len() as u64;
        let output = path.clone();
        tokio::task::spawn_blocking(move || write_match_lineup_template(&output, &data))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("比赛模板导出任务失败：{error}"))
            })??;
        Ok(MatchLineupExportSummary {
            output_path: path.to_string_lossy().to_string(),
            match_count: 0,
            lineup_count: 0,
            player_count,
        })
    }

    pub async fn export_match_lineup_data(
        &self,
        output_path: String,
        match_id: Uuid,
    ) -> ApplicationResult<MatchLineupExportSummary> {
        let path = validate_output(&output_path, "xlsx")?;
        let store = self.active_store().await?;
        let data = store.match_lineup_export_data(Some(match_id)).await?;
        let summary = MatchLineupExportSummary {
            output_path: path.to_string_lossy().to_string(),
            match_count: u64::from(data.selected_match.is_some()),
            lineup_count: data.lineups.len() as u64,
            player_count: data.players.len() as u64,
        };
        let output = path.clone();
        tokio::task::spawn_blocking(move || write_match_lineup_export(&output, &data))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("比赛数据导出任务失败：{error}"))
            })??;
        Ok(summary)
    }

    pub async fn preview_match_lineup_import(
        &self,
        input_path: String,
        mode: SpreadsheetImportMode,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let path = validate_input(&input_path, "xlsx")?;
        let parsed = tokio::task::spawn_blocking(move || read_match_lineup_workbook(&path))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("比赛 Excel 读取任务失败：{error}"))
            })??;
        let store = self.active_store().await?;
        Ok(store.preview_match_lineup_import(&parsed, mode).await?)
    }

    pub async fn read_match_lineup_import_preview(
        &self,
        batch_id: Uuid,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let store = self.active_store().await?;
        Ok(store.read_match_lineup_import_preview(batch_id).await?)
    }

    pub async fn resolve_match_lineup_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let store = self.active_store().await?;
        Ok(store
            .resolve_match_lineup_import_conflict(batch_id, resolution)
            .await?)
    }

    pub async fn commit_match_lineup_import(
        &self,
        batch_id: Uuid,
    ) -> ApplicationResult<SpreadsheetImportCommitResult> {
        let store = self.active_store().await?;
        Ok(store.commit_match_lineup_import(batch_id).await?)
    }

    pub async fn export_ai_match_package(
        &self,
        output_path: String,
        match_id: Uuid,
    ) -> ApplicationResult<AiMatchPackageSummary> {
        let path = validate_output(&output_path, "zip")?;
        let store = self.active_store().await?;
        let data = store.match_lineup_export_data(Some(match_id)).await?;
        let context = store.ai_match_package_context(match_id).await?;
        let temp = tempdir().map_err(|error| ApplicationError::Validation(error.to_string()))?;
        let workbook_path = temp.path().join("match_and_lineup.xlsx");
        let output = path.clone();
        tokio::task::spawn_blocking(move || {
            write_match_lineup_export(&workbook_path, &data)?;
            write_ai_match_package(&output, &workbook_path, &context)
        })
        .await
        .map_err(|error| ApplicationError::Validation(format!("AI 分析包导出任务失败：{error}")))?
        .map_err(ApplicationError::Spreadsheet)
    }

    pub async fn preview_ai_match_package(
        &self,
        input_path: String,
        mode: SpreadsheetImportMode,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let path = validate_input(&input_path, "zip")?;
        let temp = tempdir().map_err(|error| ApplicationError::Validation(error.to_string()))?;
        let workbook_path = temp.path().join("match_and_lineup.xlsx");
        let parsed = tokio::task::spawn_blocking(move || {
            extract_ai_match_workbook(&path, &workbook_path)?;
            read_match_lineup_workbook(&workbook_path)
        })
        .await
        .map_err(|error| {
            ApplicationError::Validation(format!("AI 分析包读取任务失败：{error}"))
        })??;
        let store = self.active_store().await?;
        Ok(store.preview_match_lineup_import(&parsed, mode).await?)
    }
}

fn validate_output(value: &str, extension: &str) -> ApplicationResult<PathBuf> {
    let path = PathBuf::from(value.trim());
    if path.as_os_str().is_empty() {
        return Err(ApplicationError::Validation("请选择输出位置".to_string()));
    }
    validate_extension(&path, extension)?;
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            ApplicationError::Validation(format!("无法创建输出目录 {}：{error}", parent.display()))
        })?;
    }
    Ok(path)
}

fn validate_input(value: &str, extension: &str) -> ApplicationResult<PathBuf> {
    let path = PathBuf::from(value.trim());
    validate_extension(&path, extension)?;
    if !path.is_file() {
        return Err(ApplicationError::Validation(format!(
            "文件不存在：{}",
            path.display()
        )));
    }
    Ok(path)
}

fn validate_extension(path: &Path, extension: &str) -> ApplicationResult<()> {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_lowercase)
        .as_deref()
        != Some(extension)
    {
        return Err(ApplicationError::Validation(format!(
            "文件必须使用 .{extension} 扩展名"
        )));
    }
    Ok(())
}
