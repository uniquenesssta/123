use crate::file_store::write_atomic;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_UNIQUE_ISSUES: usize = 500;
const MAX_MESSAGE_CHARS: usize = 4_000;
const MAX_OPERATION_CHARS: usize = 160;
const MAX_OPERATIONS_PER_ISSUE: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLogDraft {
    pub severity: String,
    pub source: String,
    pub operation: String,
    pub user_message: String,
    pub technical_message: String,
    pub occurrence_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLogEntry {
    pub id: String,
    pub severity: String,
    pub source: String,
    pub operations: Vec<String>,
    pub user_message: String,
    pub technical_message: String,
    pub occurrence_count: u64,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub app_version: String,
    #[serde(default)]
    pub occurrence_keys: Vec<String>,
}

pub struct IssueLogStore {
    path: PathBuf,
    entries: Mutex<Vec<IssueLogEntry>>,
}

impl IssueLogStore {
    pub fn new(path: PathBuf) -> Self {
        let entries = load_entries(&path);
        Self {
            path,
            entries: Mutex::new(entries),
        }
    }

    pub fn record(&self, draft: IssueLogDraft) -> Result<IssueLogEntry, String> {
        let severity = normalize_severity(&draft.severity);
        let source = truncate(draft.source.trim(), 40);
        let operation = truncate(draft.operation.trim(), MAX_OPERATION_CHARS);
        let user_message = truncate(
            &sanitize_sensitive(draft.user_message.trim()),
            MAX_MESSAGE_CHARS,
        );
        let technical_message = truncate(
            &sanitize_sensitive(draft.technical_message.trim()),
            MAX_MESSAGE_CHARS,
        );
        let fingerprint_basis = if technical_message.is_empty() {
            &user_message
        } else {
            &technical_message
        };
        let id = fingerprint(&source, fingerprint_basis);
        let now = Utc::now();
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| "问题日志暂时不可用：日志锁已损坏".to_string())?;

        let occurrence_key = draft
            .occurrence_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| truncate(value, 200));

        let result = if let Some(existing) = entries.iter_mut().find(|item| item.id == id) {
            if let Some(key) = occurrence_key.as_ref() {
                if existing.occurrence_keys.iter().any(|item| item == key) {
                    return Ok(existing.clone());
                }
                existing.occurrence_keys.push(key.clone());
                if existing.occurrence_keys.len() > 100 {
                    existing.occurrence_keys.remove(0);
                }
            }
            existing.occurrence_count = existing.occurrence_count.saturating_add(1);
            existing.last_seen_at = now;
            existing.severity = more_severe(&existing.severity, &severity).to_string();
            if !operation.is_empty() && !existing.operations.iter().any(|item| item == &operation) {
                existing.operations.push(operation);
                if existing.operations.len() > MAX_OPERATIONS_PER_ISSUE {
                    existing.operations.remove(0);
                }
            }
            if !user_message.is_empty() {
                existing.user_message = user_message;
            }
            if !technical_message.is_empty() {
                existing.technical_message = technical_message;
            }
            existing.clone()
        } else {
            let entry = IssueLogEntry {
                id,
                severity,
                source,
                operations: if operation.is_empty() {
                    Vec::new()
                } else {
                    vec![operation]
                },
                user_message,
                technical_message,
                occurrence_count: 1,
                first_seen_at: now,
                last_seen_at: now,
                app_version: env!("CARGO_PKG_VERSION").to_string(),
                occurrence_keys: occurrence_key.into_iter().collect(),
            };
            entries.push(entry.clone());
            entry
        };

        entries.sort_by(|left, right| right.last_seen_at.cmp(&left.last_seen_at));
        entries.truncate(MAX_UNIQUE_ISSUES);
        persist_entries(&self.path, &entries)?;
        Ok(result)
    }

    pub fn list(&self, limit: usize) -> Result<Vec<IssueLogEntry>, String> {
        let entries = self
            .entries
            .lock()
            .map_err(|_| "问题日志暂时不可用：日志锁已损坏".to_string())?;
        Ok(entries
            .iter()
            .take(limit.min(MAX_UNIQUE_ISSUES))
            .cloned()
            .collect())
    }

    pub fn clear(&self) -> Result<(), String> {
        let mut entries = self
            .entries
            .lock()
            .map_err(|_| "问题日志暂时不可用：日志锁已损坏".to_string())?;
        entries.clear();
        persist_entries(&self.path, &entries)
    }

    pub fn export_text(&self, output_path: &Path) -> Result<(), String> {
        let entries = self.list(MAX_UNIQUE_ISSUES)?;
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("无法创建日志导出目录 {}：{error}", parent.display()))?;
        }
        let total_occurrences: u64 = entries.iter().map(|item| item.occurrence_count).sum();
        let repeated = total_occurrences.saturating_sub(entries.len() as u64);
        let mut report = String::new();
        report.push_str("足球赛事模型平台 · 问题日志报告\n");
        report.push_str(&format!("导出时间：{}\n", Utc::now().to_rfc3339()));
        report.push_str(&format!("客户端版本：{}\n", env!("CARGO_PKG_VERSION")));
        report.push_str(&format!(
            "独立问题：{} · 总发生次数：{} · 已聚合重复：{}\n\n",
            entries.len(),
            total_occurrences,
            repeated
        ));

        for (index, item) in entries.iter().enumerate() {
            report.push_str(&format!(
                "{}. [{}] {}\n",
                index + 1,
                item.severity,
                item.user_message
            ));
            report.push_str(&format!("   问题编号：{}\n", item.id));
            report.push_str(&format!("   来源：{}\n", item.source));
            report.push_str(&format!("   发生次数：{}\n", item.occurrence_count));
            report.push_str(&format!("   首次：{}\n", item.first_seen_at.to_rfc3339()));
            report.push_str(&format!("   最近：{}\n", item.last_seen_at.to_rfc3339()));
            report.push_str(&format!("   涉及操作：{}\n", item.operations.join("、")));
            report.push_str(&format!("   技术详情：{}\n\n", item.technical_message));
        }

        write_atomic(output_path, report.as_bytes(), false)
            .map_err(|error| format!("无法导出问题日志 {}：{error}", output_path.display()))
    }
}

fn load_entries(path: &Path) -> Vec<IssueLogEntry> {
    let Ok(content) = fs::read(path) else {
        return Vec::new();
    };
    let mut entries = serde_json::from_slice::<Vec<IssueLogEntry>>(&content).unwrap_or_default();
    entries.retain(|entry| !is_user_guidance_entry(entry));
    entries
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

fn persist_entries(path: &Path, entries: &[IssueLogEntry]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建问题日志目录 {}：{error}", parent.display()))?;
    }
    let content = serde_json::to_vec_pretty(entries)
        .map_err(|error| format!("无法序列化问题日志：{error}"))?;
    write_atomic(path, &content, true)
        .map_err(|error| format!("无法写入问题日志 {}：{error}", path.display()))
}

fn normalize_severity(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "warning" => "warning".to_string(),
        "critical" => "critical".to_string(),
        _ => "error".to_string(),
    }
}

fn more_severe<'a>(left: &'a str, right: &'a str) -> &'a str {
    fn rank(value: &str) -> u8 {
        match value {
            "critical" => 3,
            "error" => 2,
            "warning" => 1,
            _ => 0,
        }
    }
    if rank(right) > rank(left) {
        right
    } else {
        left
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let mut output: String = value.chars().take(max_chars).collect();
    if value.chars().count() > max_chars {
        output.push('…');
    }
    output
}

fn sanitize_sensitive(value: &str) -> String {
    let mut output = value.to_string();
    redact_url_passwords(&mut output, "postgres://");
    redact_url_passwords(&mut output, "postgresql://");
    output
}

fn redact_url_passwords(output: &mut String, scheme: &str) {
    let mut search_from = 0usize;
    while let Some(relative_start) = output[search_from..].find(scheme) {
        let start = search_from + relative_start + scheme.len();
        let Some(relative_at) = output[start..].find('@') else {
            break;
        };
        let at = start + relative_at;
        let Some(relative_colon) = output[start..at].find(':') else {
            search_from = at + 1;
            continue;
        };
        let password_start = start + relative_colon + 1;
        output.replace_range(password_start..at, "***");
        search_from = password_start + 3;
    }
}

fn fingerprint(source: &str, message: &str) -> String {
    let normalized = normalize_for_fingerprint(message);
    let fingerprint_input = format!("{source}|{normalized}");
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in fingerprint_input.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("ISSUE-{hash:016X}")
}

fn normalize_for_fingerprint(message: &str) -> String {
    let lowercase = message.to_lowercase();
    let characters: Vec<char> = lowercase.chars().collect();
    let mut output = String::with_capacity(lowercase.len());
    let mut previous_was_space = false;
    let mut previous_was_digit_marker = false;
    let mut index = 0usize;

    while index < characters.len() {
        let character = characters[index];
        if character.is_ascii_hexdigit() || character == '-' {
            let start = index;
            let mut hyphen_count = 0usize;
            while index < characters.len()
                && (characters[index].is_ascii_hexdigit() || characters[index] == '-')
            {
                if characters[index] == '-' {
                    hyphen_count += 1;
                }
                index += 1;
            }
            if index - start >= 20 && hyphen_count >= 2 {
                output.push_str("<id>");
                previous_was_space = false;
                previous_was_digit_marker = false;
                continue;
            }
            for value in &characters[start..index] {
                append_normalized_character(
                    &mut output,
                    *value,
                    &mut previous_was_space,
                    &mut previous_was_digit_marker,
                );
            }
            continue;
        }

        append_normalized_character(
            &mut output,
            character,
            &mut previous_was_space,
            &mut previous_was_digit_marker,
        );
        index += 1;
    }

    output.trim().to_string()
}

fn append_normalized_character(
    output: &mut String,
    character: char,
    previous_was_space: &mut bool,
    previous_was_digit_marker: &mut bool,
) {
    if character.is_ascii_digit() {
        if !*previous_was_digit_marker {
            output.push('#');
            *previous_was_digit_marker = true;
        }
        *previous_was_space = false;
    } else if character.is_whitespace() {
        if !*previous_was_space {
            output.push(' ');
            *previous_was_space = true;
        }
        *previous_was_digit_marker = false;
    } else {
        output.push(character);
        *previous_was_space = false;
        *previous_was_digit_marker = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn similar_messages_share_fingerprint() {
        assert_eq!(
            fingerprint("backend", "任务 123 在 2026-07-13 失败"),
            fingerprint("backend", "任务 456 在 2026-07-14 失败")
        );
    }

    #[test]
    fn different_uuids_share_fingerprint() {
        assert_eq!(
            fingerprint(
                "backend",
                "比赛 550e8400-e29b-41d4-a716-446655440000 不存在"
            ),
            fingerprint(
                "backend",
                "比赛 4d36e967-e325-11ce-bfc1-08002be10318 不存在"
            )
        );
    }

    #[test]
    fn different_columns_do_not_share_fingerprint() {
        assert_ne!(
            fingerprint("backend", "column observation.created_at does not exist"),
            fingerprint("backend", "column observation.updated_at does not exist")
        );
    }

    #[test]
    fn password_is_redacted() {
        assert_eq!(
            sanitize_sensitive("连接 postgres://user:secret@localhost/db 失败"),
            "连接 postgres://user:***@localhost/db 失败"
        );
        assert_eq!(
            sanitize_sensitive("连接 postgresql://user:secret@localhost/db 失败"),
            "连接 postgresql://user:***@localhost/db 失败"
        );
    }
}
