use crate::{ApplicationError, ApplicationResult};
use football_domain::{MatchReviewPackageData, MatchReviewPackagePreview};
use football_spreadsheet_io::{read_match_review_package, write_match_review_package};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(super) fn validate_path(value: &str, output: bool) -> ApplicationResult<PathBuf> {
    let path = PathBuf::from(value.trim());
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
        != Some("xlsx")
    {
        return Err(ApplicationError::Validation(
            "赛后复盘资料包必须使用 .xlsx 扩展名".to_string(),
        ));
    }
    if output {
        if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|error| {
                ApplicationError::Validation(format!(
                    "无法创建输出目录 {}：{error}",
                    parent.display()
                ))
            })?;
        }
    } else if !path.is_file() {
        return Err(ApplicationError::Validation(format!(
            "文件不存在：{}",
            path.display()
        )));
    }
    Ok(path)
}

pub(super) async fn write_package(
    path: PathBuf,
    data: MatchReviewPackageData,
) -> ApplicationResult<()> {
    tokio::task::spawn_blocking(move || write_match_review_package(&path, &data))
        .await
        .map_err(|error| {
            ApplicationError::Validation(format!("赛后复盘资料包导出任务失败：{error}"))
        })??;
    Ok(())
}

pub(super) async fn read_package(path: PathBuf) -> ApplicationResult<MatchReviewPackagePreview> {
    tokio::task::spawn_blocking(move || read_match_review_package(&path))
        .await
        .map_err(|error| {
            ApplicationError::Validation(format!("赛后复盘资料包读取任务失败：{error}"))
        })?
        .map_err(ApplicationError::from)
}

pub(super) fn sha256_file(path: &Path) -> ApplicationResult<String> {
    let bytes = std::fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
