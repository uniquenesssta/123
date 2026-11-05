use super::{sanitize_sensitive, IssueLogEntry};
use crate::file_store::write_atomic;
use std::{fs, path::Path};
pub(super) fn load_entries(path: &Path) -> Result<Vec<IssueLogEntry>, String> {
    let content = match fs::read(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(format!("问题日志读取失败，原文件已保留：{error}")),
    };
    let mut entries: Vec<IssueLogEntry> = serde_json::from_slice(&content)
        .map_err(|_| "问题日志文件格式损坏，原文件已保留；请修复后重新读取".to_string())?;
    for entry in &mut entries {
        if !["error", "warning", "critical"].contains(&entry.severity.as_str())
            || entry.occurrence_count == 0
        {
            return Err("问题日志记录格式异常，原文件已保留".into());
        }
        entry.id = sanitize_sensitive(&entry.id);
        entry.source = sanitize_sensitive(&entry.source);
        entry.user_message = sanitize_sensitive(&entry.user_message);
        entry.technical_message = sanitize_sensitive(&entry.technical_message);
        entry.app_version = sanitize_sensitive(&entry.app_version);
        for operation in &mut entry.operations {
            *operation = sanitize_sensitive(operation)
        }
        for key in &mut entry.occurrence_keys {
            *key = sanitize_sensitive(key)
        }
    }
    entries.retain(|entry| !is_user_guidance_entry(entry));
    entries.sort_by(|a, b| b.last_seen_at.cmp(&a.last_seen_at));
    entries.truncate(super::MAX_UNIQUE_ISSUES);
    Ok(entries)
}
fn is_user_guidance_entry(entry: &IssueLogEntry) -> bool {
    let message = if entry.technical_message.trim().is_empty() {
        entry.user_message.trim()
    } else {
        entry.technical_message.trim()
    };
    matches!(
        message,
        "请选择需要复盘的比赛" | "请先选择比赛" | "当前没有可载入的复盘比赛"
    )
}

pub(super) fn persist_entries(path: &Path, entries: &[IssueLogEntry]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建问题日志目录 {}：{error}", parent.display()))?;
    }
    let content = serde_json::to_vec_pretty(entries)
        .map_err(|error| format!("无法序列化问题日志：{error}"))?;
    write_atomic(path, &content, true)
        .map_err(|error| format!("无法写入问题日志 {}：{error}", path.display()))
}
