use crate::diagnostic_redaction::redact_text as sanitize_sensitive;
use aggregation::*;
use storage::{load_entries, persist_entries};
mod aggregation;
mod report;
mod storage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_UNIQUE_ISSUES: usize = 500;
const MAX_MESSAGE_CHARS: usize = 4_000;
const MAX_OPERATION_CHARS: usize = 160;
const MAX_OPERATIONS_PER_ISSUE: usize = 8;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IssueLogDraft {
    pub severity: String,
    pub source: String,
    pub operation: String,
    pub user_message: String,
    pub technical_message: String,
    pub occurrence_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    state: Mutex<StoreState>,
}
struct StoreState {
    entries: Vec<IssueLogEntry>,
    load_error: Option<String>,
}
impl IssueLogStore {
    pub fn new(path: PathBuf) -> Self {
        let state = match load_entries(&path) {
            Ok(entries) => StoreState {
                entries,
                load_error: None,
            },
            Err(error) => StoreState {
                entries: vec![],
                load_error: Some(error),
            },
        };
        Self {
            path,
            state: Mutex::new(state),
        }
    }
    fn lock_loaded(&self) -> Result<std::sync::MutexGuard<'_, StoreState>, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "问题日志暂时不可用：日志锁已损坏".to_string())?;
        if state.load_error.is_some() {
            match load_entries(&self.path) {
                Ok(entries) => {
                    state.entries = entries;
                    state.load_error = None
                }
                Err(error) => {
                    state.load_error = Some(error.clone());
                    return Err(error);
                }
            }
        }
        Ok(state)
    }
    pub fn record(&self, draft: IssueLogDraft) -> Result<IssueLogEntry, String> {
        let severity = normalize_severity(&draft.severity);
        let source = truncate(&sanitize_sensitive(draft.source.trim()), 40);
        let operation = truncate(
            &sanitize_sensitive(draft.operation.trim()),
            MAX_OPERATION_CHARS,
        );
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
        let mut state = self.lock_loaded()?;
        let mut entries = state.entries.clone();

        let occurrence_key = draft
            .occurrence_key
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| truncate(&sanitize_sensitive(value), 200));

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
        state.entries = entries;
        Ok(result)
    }

    pub fn list(&self, limit: usize) -> Result<Vec<IssueLogEntry>, String> {
        let state = self.lock_loaded()?;
        Ok(state
            .entries
            .iter()
            .take(limit.min(MAX_UNIQUE_ISSUES))
            .cloned()
            .collect())
    }
    pub fn clear(&self) -> Result<(), String> {
        self.clear_snapshot(None)
    }
    pub fn clear_snapshot(&self, expected: Option<&[IssueLogEntry]>) -> Result<(), String> {
        let mut state = self.lock_loaded()?;
        check_snapshot(&state.entries, expected)?;
        persist_entries(&self.path, &[])?;
        state.entries.clear();
        Ok(())
    }
    pub fn export_text(&self, output_path: &Path) -> Result<(), String> {
        self.export_snapshot(output_path, None)
    }
    pub fn export_snapshot(
        &self,
        output_path: &Path,
        expected: Option<&[IssueLogEntry]>,
    ) -> Result<(), String> {
        let entries = {
            let state = self.lock_loaded()?;
            check_snapshot(&state.entries, expected)?;
            state.entries.clone()
        };
        report::export_text(output_path, &entries)
    }
}
fn check_snapshot(
    current: &[IssueLogEntry],
    expected: Option<&[IssueLogEntry]>,
) -> Result<(), String> {
    if expected.is_some_and(|entries| entries != current) {
        return Err("问题日志集合已变化，请刷新后重新核对并确认".to_string());
    }
    Ok(())
}
#[cfg(test)]
mod reliability_tests;

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
            "连接 postgres://[REDACTED]@localhost/db 失败"
        );
        assert_eq!(
            sanitize_sensitive("连接 postgresql://user:secret@localhost/db 失败"),
            "连接 postgresql://[REDACTED]@localhost/db 失败"
        );
    }
}
