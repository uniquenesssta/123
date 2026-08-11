use crate::{
    use_cases::exchange::file_validation::spreadsheet::validate_json_path,
    ApplicationError, ApplicationResult,
};
use chrono::Utc;
use football_domain::{
    TeamPackageImportPreview, TeamPackagePreviewExportSummary, TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT,
};
use serde_json::json;

pub(crate) async fn execute(
    output_path: String,
    preview: TeamPackageImportPreview,
) -> ApplicationResult<TeamPackagePreviewExportSummary> {
    let path = validate_json_path(&output_path)?;
    let team_row_count = preview
        .team_preview
        .as_ref()
        .map(|value| value.rows.len() as u64)
        .unwrap_or(0);
    let player_row_count = preview
        .player_preview
        .as_ref()
        .map(|value| value.rows.len() as u64)
        .unwrap_or(0);
    let exported_row_count = team_row_count + player_row_count;
    let payload = json!({
        "format_version": TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT,
        "exported_at": Utc::now(),
        "source": {
            "file_name": preview.source_file_name,
            "sha256": preview.source_sha256,
        },
        "summary": {
            "team_row_count": team_row_count,
            "player_row_count": player_row_count,
            "exported_row_count": exported_row_count,
            "coverage": preview.coverage,
        },
        "team_preview": preview.team_preview,
        "player_preview": preview.player_preview,
    });
    let output = path.clone();
    tokio::task::spawn_blocking(move || -> ApplicationResult<()> {
        let bytes = serde_json::to_vec_pretty(&payload).map_err(|error| {
            ApplicationError::Validation(format!("完整预检 JSON 序列化失败：{error}"))
        })?;
        std::fs::write(&output, bytes).map_err(|error| {
            ApplicationError::Validation(format!(
                "完整预检 JSON 写入失败 {}：{error}",
                output.display()
            ))
        })?;
        Ok(())
    })
    .await
    .map_err(|error| ApplicationError::Validation(format!("完整预检 JSON 导出任务失败：{error}")))??;
    Ok(TeamPackagePreviewExportSummary {
        output_path: path.to_string_lossy().to_string(),
        format_version: TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT.to_string(),
        exported_row_count,
    })
}
