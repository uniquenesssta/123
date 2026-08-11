use crate::{ApplicationError, ApplicationResult};
use std::path::{Path, PathBuf};

pub(crate) fn validate_output(value: &str, extension: &str) -> ApplicationResult<PathBuf> {
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

pub(crate) fn validate_input(value: &str, extension: &str) -> ApplicationResult<PathBuf> {
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
