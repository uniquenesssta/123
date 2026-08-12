use crate::{ApplicationError, ApplicationResult};
use std::path::PathBuf;

pub(crate) fn validate_json_path(value: &str) -> ApplicationResult<PathBuf> {
    validate_output_path(value, "json", "请选择 JSON 输出位置")
}

pub(crate) fn validate_xlsx_path(value: &str) -> ApplicationResult<PathBuf> {
    validate_output_path(value, "xlsx", "请选择 Excel 输出位置")
}

pub(crate) fn validate_existing_xlsx_path(value: &str) -> ApplicationResult<PathBuf> {
    let path = validate_xlsx_path(value)?;
    if !path.is_file() {
        return Err(ApplicationError::Validation(format!(
            "Excel 文件不存在：{}",
            path.display()
        )));
    }
    Ok(path)
}

fn validate_output_path(
    value: &str,
    extension: &str,
    empty_message: &str,
) -> ApplicationResult<PathBuf> {
    let path = PathBuf::from(value.trim());
    if path.as_os_str().is_empty() {
        return Err(ApplicationError::Validation(empty_message.to_string()));
    }
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_lowercase)
        .as_deref()
        != Some(extension)
    {
        return Err(ApplicationError::Validation(format!(
            "输出文件必须使用 .{extension} 扩展名"
        )));
    }
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
