use crate::{ApplicationError, ApplicationResult};
use football_domain::ApiWorkspaceAttachment;
use football_spreadsheet_io::read_workbook_for_api;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const MAX_ATTACHMENT_FILES: usize = 5;
const MAX_ATTACHMENT_TOTAL_BYTES: u64 = 5 * 1024 * 1024;
const MAX_TEXT_ATTACHMENT_BYTES: usize = 1_500_000;

pub async fn execute(paths: Vec<String>) -> ApplicationResult<Vec<ApiWorkspaceAttachment>> {
    if paths.len() > MAX_ATTACHMENT_FILES {
        return Err(ApplicationError::Validation(format!(
            "一次最多选择{MAX_ATTACHMENT_FILES}个附件"
        )));
    }
    let mut total = 0u64;
    let mut attachments = Vec::with_capacity(paths.len());
    for raw_path in paths {
        let path = PathBuf::from(raw_path.trim());
        if !path.is_file() {
            return Err(ApplicationError::Validation(format!(
                "附件不存在：{}",
                path.display()
            )));
        }
        let metadata = std::fs::metadata(&path)?;
        total = total.saturating_add(metadata.len());
        if total > MAX_ATTACHMENT_TOTAL_BYTES {
            return Err(ApplicationError::Validation(
                "附件总大小不能超过5 MiB".to_string(),
            ));
        }
        attachments.push(read_attachment(&path, metadata.len()).await?);
    }
    Ok(attachments)
}

async fn read_attachment(
    path: &Path,
    original_size: u64,
) -> ApplicationResult<ApiWorkspaceAttachment> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| ApplicationError::Validation("附件缺少扩展名".to_string()))?;
    let (media_type, mut content) = match extension.as_str() {
        "txt" => ("text/plain", read_utf8(path)?),
        "md" => ("text/markdown", read_utf8(path)?),
        "json" => {
            let text = read_utf8(path)?;
            let value: Value = serde_json::from_str(&text)?;
            ("application/json", serde_json::to_string_pretty(&value)?)
        }
        "csv" => ("text/csv", read_utf8(path)?),
        "tsv" => ("text/tab-separated-values", read_utf8(path)?),
        "xlsx" => {
            let path = path.to_path_buf();
            let content = tokio::task::spawn_blocking(move || read_workbook_for_api(&path))
                .await
                .map_err(|error| {
                    ApplicationError::Validation(format!("Excel附件读取任务失败：{error}"))
                })??;
            (
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                content,
            )
        }
        _ => {
            return Err(ApplicationError::Validation(format!(
                "不支持的附件类型：.{extension}；仅支持 txt、md、json、csv、tsv、xlsx"
            )))
        }
    };
    let truncated = content.len() > MAX_TEXT_ATTACHMENT_BYTES;
    if truncated {
        truncate_utf8_bytes(&mut content, MAX_TEXT_ATTACHMENT_BYTES);
        content.push_str("\n[attachment content truncated by the desktop client]\n");
    }
    let content_sha256 = hex::encode(Sha256::digest(content.as_bytes()));
    Ok(ApiWorkspaceAttachment {
        name: path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("attachment")
            .to_string(),
        media_type: media_type.to_string(),
        content,
        content_sha256,
        original_size_bytes: original_size,
        truncated,
    })
}

fn read_utf8(path: &Path) -> ApplicationResult<String> {
    let bytes = std::fs::read(path)?;
    String::from_utf8(bytes)
        .map_err(|_| ApplicationError::Validation(format!("附件不是UTF-8文本：{}", path.display())))
}

fn truncate_utf8_bytes(value: &mut String, max_bytes: usize) {
    if value.len() <= max_bytes {
        return;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    value.truncate(boundary);
}
