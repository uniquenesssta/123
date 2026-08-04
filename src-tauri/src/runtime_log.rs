use chrono::{SecondsFormat, Utc};
use serde_json::{json, Map, Value};
use std::collections::{hash_map::DefaultHasher, HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

const LOG_DIRECTORY_NAME: &str = "logs";
const RUNTIME_ROOT_ENV: &str = "FOOTBALL_RUNTIME_ROOT";
const LEGACY_PROJECT_ROOT_ENV: &str = "FOOTBALL_PROJECT_ROOT";
const LOG_FILE_PREFIX: &str = "football-runtime";
const DUPLICATE_WINDOW: Duration = Duration::from_secs(5);
const MAX_TRACKED_DUPLICATE_EVENTS: usize = 256;
const MAX_STRING_CHARS: usize = 32_768;

pub struct RuntimeLogStore {
    path: PathBuf,
    session_id: String,
    state: Mutex<RuntimeLogState>,
}

#[derive(Default)]
struct RuntimeLogState {
    sequence: u64,
    duplicates: HashMap<u64, DuplicateState>,
}

struct DuplicateState {
    fingerprint: u64,
    level: String,
    subsystem: String,
    event: String,
    first_trace_id: Option<String>,
    last_trace_id: Option<String>,
    first_seen_utc: String,
    last_seen_utc: String,
    last_seen_instant: Instant,
    suppressed_count: u64,
}

impl RuntimeLogStore {
    pub fn discover(fallback_dir: &Path) -> Self {
        let session_id = Uuid::new_v4().to_string();
        let mut candidates = runtime_root_candidates(fallback_dir);
        candidates.push(fallback_dir.to_path_buf());
        let mut seen = HashSet::new();
        for directory in candidates {
            let normalized = directory.to_string_lossy().into_owned();
            if !seen.insert(normalized) {
                continue;
            }
            if let Ok(path) = create_session_log(&directory, &session_id) {
                return Self {
                    path,
                    session_id,
                    state: Mutex::new(RuntimeLogState::default()),
                };
            }
        }

        let path = fallback_dir
            .join(LOG_DIRECTORY_NAME)
            .join(session_log_filename(&session_id, 0));
        Self {
            path,
            session_id,
            state: Mutex::new(RuntimeLogState::default()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn relative_display_path(&self) -> String {
        let filename = self
            .path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("football-runtime.jsonl");
        format!(r".\logs\{filename}")
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn record(
        &self,
        level: &str,
        subsystem: &str,
        event: &str,
        trace_id: Option<&str>,
        details: Value,
    ) -> Result<(), String> {
        let normalized_level = normalize_level(level);
        let sanitized_details = sanitize_value(details);
        let fingerprint = event_fingerprint(normalized_level, subsystem, event, &sanitized_details);
        let now = Utc::now();
        let now_text = now.to_rfc3339_opts(SecondsFormat::Millis, true);
        let now_instant = Instant::now();
        let mut state = self
            .state
            .lock()
            .map_err(|_| "运行日志锁已损坏".to_string())?;

        self.flush_expired_duplicate_summaries(&mut state, now_instant)?;
        if should_deduplicate(normalized_level, event) {
            if let Some(duplicate) = state.duplicates.get_mut(&fingerprint) {
                duplicate.suppressed_count += 1;
                duplicate.last_seen_utc = now_text;
                duplicate.last_seen_instant = now_instant;
                duplicate.last_trace_id = trace_id.map(str::to_string);
                return Ok(());
            }
        }

        state.sequence += 1;
        self.write_entry(
            state.sequence,
            &now_text,
            normalized_level,
            subsystem,
            event,
            trace_id,
            sanitized_details,
        )?;

        if should_deduplicate(normalized_level, event) {
            state.duplicates.insert(
                fingerprint,
                DuplicateState {
                    fingerprint,
                    level: normalized_level.to_string(),
                    subsystem: subsystem.to_string(),
                    event: event.to_string(),
                    first_trace_id: trace_id.map(str::to_string),
                    last_trace_id: trace_id.map(str::to_string),
                    first_seen_utc: now_text.clone(),
                    last_seen_utc: now_text,
                    last_seen_instant: now_instant,
                    suppressed_count: 0,
                },
            );
            self.trim_duplicate_states(&mut state)?;
        }
        Ok(())
    }

    fn flush_expired_duplicate_summaries(
        &self,
        state: &mut RuntimeLogState,
        now: Instant,
    ) -> Result<(), String> {
        let mut expired = state
            .duplicates
            .iter()
            .filter(|(_, duplicate)| {
                now.duration_since(duplicate.last_seen_instant) > DUPLICATE_WINDOW
            })
            .map(|(fingerprint, duplicate)| (*fingerprint, duplicate.last_seen_instant))
            .collect::<Vec<_>>();
        expired.sort_by_key(|(_, last_seen)| *last_seen);
        for (fingerprint, _) in expired {
            if let Some(duplicate) = state.duplicates.remove(&fingerprint) {
                self.write_duplicate_summary(state, duplicate)?;
            }
        }
        Ok(())
    }

    fn trim_duplicate_states(&self, state: &mut RuntimeLogState) -> Result<(), String> {
        while state.duplicates.len() > MAX_TRACKED_DUPLICATE_EVENTS {
            let Some(fingerprint) = state
                .duplicates
                .iter()
                .min_by_key(|(_, duplicate)| duplicate.last_seen_instant)
                .map(|(fingerprint, _)| *fingerprint)
            else {
                break;
            };
            if let Some(duplicate) = state.duplicates.remove(&fingerprint) {
                self.write_duplicate_summary(state, duplicate)?;
            }
        }
        Ok(())
    }

    fn write_duplicate_summary(
        &self,
        state: &mut RuntimeLogState,
        duplicate: DuplicateState,
    ) -> Result<(), String> {
        if duplicate.suppressed_count == 0 {
            return Ok(());
        }
        state.sequence += 1;
        let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        self.write_entry(
            state.sequence,
            &timestamp,
            "info",
            "runtime_log",
            "duplicate_events_suppressed",
            duplicate.last_trace_id.as_deref(),
            json!({
                "original_level": duplicate.level,
                "original_subsystem": duplicate.subsystem,
                "original_event": duplicate.event,
                "original_fingerprint": duplicate.fingerprint,
                "suppressed_count": duplicate.suppressed_count,
                "first_trace_id": duplicate.first_trace_id,
                "last_trace_id": duplicate.last_trace_id,
                "first_seen_utc": duplicate.first_seen_utc,
                "last_seen_utc": duplicate.last_seen_utc,
                "dedup_window_ms": DUPLICATE_WINDOW.as_millis(),
            }),
        )
    }

    fn write_entry(
        &self,
        sequence: u64,
        timestamp_utc: &str,
        level: &str,
        subsystem: &str,
        event: &str,
        trace_id: Option<&str>,
        details: Value,
    ) -> Result<(), String> {
        ensure_log_file(&self.path)?;
        let entry = json!({
            "timestamp_utc": timestamp_utc,
            "session_id": self.session_id,
            "sequence": sequence,
            "level": level,
            "subsystem": subsystem,
            "event": event,
            "app_version": env!("CARGO_PKG_VERSION"),
            "trace_id": trace_id,
            "details": details,
        });
        let mut line =
            serde_json::to_vec(&entry).map_err(|error| format!("无法序列化运行日志：{error}"))?;
        line.push(b'\n');
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|error| format!("无法打开运行日志 {}：{error}", self.path.display()))?;
        file.write_all(&line)
            .and_then(|_| file.flush())
            .map_err(|error| format!("无法写入运行日志 {}：{error}", self.path.display()))
    }
}

impl Drop for RuntimeLogStore {
    fn drop(&mut self) {
        let pending = self
            .state
            .get_mut()
            .ok()
            .map(|state| {
                let mut duplicates = state
                    .duplicates
                    .drain()
                    .map(|(_, value)| value)
                    .collect::<Vec<_>>();
                duplicates.sort_by_key(|duplicate| duplicate.last_seen_instant);
                duplicates
                    .into_iter()
                    .filter(|duplicate| duplicate.suppressed_count > 0)
                    .map(|duplicate| {
                        state.sequence += 1;
                        (state.sequence, duplicate)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for (sequence, duplicate) in pending {
            let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
            let _ = self.write_entry(
                sequence,
                &timestamp,
                "info",
                "runtime_log",
                "duplicate_events_suppressed",
                duplicate.last_trace_id.as_deref(),
                json!({
                    "original_level": duplicate.level,
                    "original_subsystem": duplicate.subsystem,
                    "original_event": duplicate.event,
                    "original_fingerprint": duplicate.fingerprint,
                    "suppressed_count": duplicate.suppressed_count,
                    "first_trace_id": duplicate.first_trace_id,
                    "last_trace_id": duplicate.last_trace_id,
                    "first_seen_utc": duplicate.first_seen_utc,
                    "last_seen_utc": duplicate.last_seen_utc,
                    "dedup_window_ms": DUPLICATE_WINDOW.as_millis(),
                }),
            );
        }
    }
}

fn diagnostic_noise_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "encrypted_content" | "prompt_cache_key" | "safety_identifier"
    )
}

fn runtime_root_candidates(fallback_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    append_configured_root(&mut candidates, RUNTIME_ROOT_ENV);
    append_configured_root(&mut candidates, LEGACY_PROJECT_ROOT_ENV);
    append_ancestor_project_candidates(&mut candidates, fallback_dir, 16);
    if let Ok(current) = std::env::current_dir() {
        append_ancestor_project_candidates(&mut candidates, &current, 16);
    }
    if let Ok(executable) = std::env::current_exe() {
        if let Some(executable_directory) = executable.parent() {
            append_ancestor_project_candidates(&mut candidates, executable_directory, 8);
            if !is_cargo_build_directory(executable_directory) {
                candidates.push(executable_directory.to_path_buf());
            }
        }
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    append_ancestor_project_candidates(&mut candidates, &manifest_dir, 6);
    candidates
}

fn append_configured_root(candidates: &mut Vec<PathBuf>, variable: &str) {
    if let Ok(explicit) = std::env::var(variable) {
        let path = PathBuf::from(explicit);
        if !path.as_os_str().is_empty() {
            candidates.push(path);
        }
    }
}

fn is_cargo_build_directory(directory: &Path) -> bool {
    directory.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        value == ".cargo-target" || value == "target"
    })
}

fn append_ancestor_project_candidates(
    candidates: &mut Vec<PathBuf>,
    directory: &Path,
    maximum_depth: usize,
) {
    for ancestor in directory.ancestors().take(maximum_depth) {
        append_project_candidates(candidates, ancestor);
    }
}

fn append_project_candidates(candidates: &mut Vec<PathBuf>, directory: &Path) {
    if directory.join("README.md").is_file() && contains_source_directory(directory) {
        candidates.push(directory.to_path_buf());
    }
    if directory.join("package.json").is_file() && directory.join("Cargo.toml").is_file() {
        if let Some(parent) = directory.parent() {
            candidates.push(parent.to_path_buf());
        }
    }
}

fn contains_source_directory(root: &Path) -> bool {
    fs::read_dir(root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.filter_map(Result::ok))
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.path())
        .any(|path| path.join("package.json").is_file() && path.join("Cargo.toml").is_file())
}

fn create_session_log(directory: &Path, session_id: &str) -> Result<PathBuf, String> {
    let log_directory = directory.join(LOG_DIRECTORY_NAME);
    fs::create_dir_all(&log_directory)
        .map_err(|error| format!("无法创建运行日志目录 {}：{error}", log_directory.display()))?;
    for collision_index in 0..100u16 {
        let path = log_directory.join(session_log_filename(session_id, collision_index));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(_) => return Ok(path),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!("无法创建运行日志 {}：{error}", path.display()));
            }
        }
    }
    Err(format!(
        "无法在 {} 创建唯一运行日志文件",
        log_directory.display()
    ))
}

fn session_log_filename(session_id: &str, collision_index: u16) -> String {
    let timestamp = Utc::now().format("%Y%m%dT%H%M%S%.3fZ").to_string();
    let process_id = std::process::id();
    let suffix = if collision_index == 0 {
        String::new()
    } else {
        format!("-{collision_index}")
    };
    format!(
        "{LOG_FILE_PREFIX}-{timestamp}-pid{process_id}-{}{suffix}.jsonl",
        &session_id[..8]
    )
}

fn ensure_log_file(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("无法创建运行日志目录 {}：{error}", parent.display()))?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map(|_| ())
        .map_err(|error| format!("无法创建运行日志 {}：{error}", path.display()))
}

fn should_deduplicate(level: &str, event: &str) -> bool {
    matches!(level, "debug" | "info" | "warning")
        && !matches!(
            event,
            "operation_started" | "operation_completed" | "operation_failed"
        )
}

fn event_fingerprint(level: &str, subsystem: &str, event: &str, details: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    level.hash(&mut hasher);
    subsystem.hash(&mut hasher);
    event.hash(&mut hasher);
    let normalized = normalize_for_fingerprint(details);
    serde_json::to_vec(&normalized)
        .unwrap_or_default()
        .hash(&mut hasher);
    hasher.finish()
}

fn normalize_for_fingerprint(value: &Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut output = Map::new();
            for (key, value) in object {
                if volatile_diagnostic_key(key) {
                    continue;
                }
                output.insert(key.clone(), normalize_for_fingerprint(value));
            }
            Value::Object(output)
        }
        Value::Array(values) => {
            Value::Array(values.iter().map(normalize_for_fingerprint).collect())
        }
        other => other.clone(),
    }
}

fn volatile_diagnostic_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "duration_ms"
            | "latency_ms"
            | "elapsed_ms"
            | "timestamp"
            | "timestamp_utc"
            | "started_at"
            | "completed_at"
    )
}

fn normalize_level(level: &str) -> &'static str {
    match level.trim().to_ascii_lowercase().as_str() {
        "debug" => "debug",
        "warning" | "warn" => "warning",
        "error" => "error",
        "critical" => "critical",
        _ => "info",
    }
}

fn sanitize_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut output = Map::new();
            for (key, value) in object {
                if diagnostic_noise_key(&key) {
                    continue;
                }
                if sensitive_key(&key) {
                    output.insert(key, Value::String("[REDACTED]".to_string()));
                } else {
                    output.insert(key, sanitize_value(value));
                }
            }
            Value::Object(output)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(sanitize_value).collect()),
        Value::String(value) => Value::String(sanitize_string(&value)),
        other => other,
    }
}

fn sensitive_key(key: &str) -> bool {
    matches!(
        key.trim().to_ascii_lowercase().as_str(),
        "authorization"
            | "api_key"
            | "apikey"
            | "access_token"
            | "refresh_token"
            | "password"
            | "secret"
            | "credential"
            | "credentials"
    )
}

fn sanitize_string(value: &str) -> String {
    let mut output = value.to_string();
    redact_url_passwords(&mut output, "postgres://");
    redact_url_passwords(&mut output, "postgresql://");
    redact_bearer_tokens(&mut output);
    let char_count = output.chars().count();
    if char_count > MAX_STRING_CHARS {
        let mut truncated = output.chars().take(MAX_STRING_CHARS).collect::<String>();
        truncated.push_str(&format!("…[truncated original_chars={char_count}]"));
        truncated
    } else {
        output
    }
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

fn redact_bearer_tokens(output: &mut String) {
    for prefix in ["Bearer ", "bearer "] {
        let mut search_from = 0usize;
        while let Some(relative_start) = output[search_from..].find(prefix) {
            let token_start = search_from + relative_start + prefix.len();
            let token_end = output[token_start..]
                .find(|ch: char| ch.is_whitespace() || matches!(ch, '"' | '\'' | ',' | '}'))
                .map(|offset| token_start + offset)
                .unwrap_or(output.len());
            if token_end > token_start {
                output.replace_range(token_start..token_end, "[REDACTED]");
                search_from = token_start + "[REDACTED]".len();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_store(path: PathBuf) -> RuntimeLogStore {
        RuntimeLogStore {
            path,
            session_id: "00000000-0000-0000-0000-000000000000".to_string(),
            state: Mutex::new(RuntimeLogState::default()),
        }
    }

    #[test]
    fn sensitive_values_are_redacted_without_hiding_token_counts() {
        let value = sanitize_value(json!({
            "api_key": "secret-value",
            "max_output_tokens": 1024,
            "database_url": "postgres://user:password@localhost/db",
            "header": "Bearer abcdef"
        }));
        assert_eq!(value["api_key"], "[REDACTED]");
        assert_eq!(value["max_output_tokens"], 1024);
        assert_eq!(value["database_url"], "postgres://user:***@localhost/db");
        assert_eq!(value["header"], "Bearer [REDACTED]");
    }

    #[test]
    fn every_session_creates_a_distinct_file_in_logs_directory() {
        let directory = tempdir().expect("temp directory");
        let first = create_session_log(directory.path(), "aaaaaaaa-0000-0000-0000-000000000000")
            .expect("first log");
        let second = create_session_log(directory.path(), "bbbbbbbb-0000-0000-0000-000000000000")
            .expect("second log");
        assert_ne!(first, second);
        let logs_directory = directory.path().join("logs");
        assert_eq!(first.parent(), Some(logs_directory.as_path()));
        assert_eq!(
            first.extension().and_then(|value| value.to_str()),
            Some("jsonl")
        );
    }

    #[test]
    fn user_facing_path_is_relative_to_logs_directory() {
        let store = test_store(PathBuf::from("any-root/logs/session.jsonl"));
        assert_eq!(store.relative_display_path(), r".\logs\session.jsonl");
    }

    #[test]
    fn repeated_diagnostics_are_suppressed_and_summarized() {
        let directory = tempdir().expect("temp directory");
        let path = directory.path().join("runtime.jsonl");
        let store = test_store(path.clone());
        for _ in 0..3 {
            store
                .record(
                    "info",
                    "frontend.operation",
                    "operation_diagnostic",
                    Some("trace"),
                    json!({"operation": "searchable_select", "state": "opened"}),
                )
                .expect("record repeated event");
        }
        store
            .record(
                "info",
                "frontend.operation",
                "operation_navigation",
                Some("next"),
                json!({"page": "players"}),
            )
            .expect("record distinct event");
        drop(store);
        let lines = fs::read_to_string(path).expect("read log");
        let entries = lines
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("valid JSONL"))
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[2]["event"], "duplicate_events_suppressed");
        assert_eq!(entries[2]["details"]["suppressed_count"], 2);
    }

    #[test]
    fn operation_lifecycle_events_keep_trace_pairs() {
        let directory = tempdir().expect("temp directory");
        let path = directory.path().join("runtime.jsonl");
        let store = test_store(path.clone());
        for index in 0..3 {
            let trace = format!("operation-{index}");
            store
                .record(
                    "info",
                    "frontend.operation",
                    "operation_started",
                    Some(&trace),
                    json!({"operation": "save_workspace_state", "context": {"args": null}}),
                )
                .expect("record operation start");
            store
                .record(
                    "info",
                    "frontend.operation",
                    "operation_completed",
                    Some(&trace),
                    json!({"operation": "save_workspace_state", "duration_ms": index}),
                )
                .expect("record operation completion");
        }
        drop(store);
        let entries = fs::read_to_string(path)
            .expect("read log")
            .lines()
            .map(|line| serde_json::from_str::<Value>(line).expect("valid JSONL"))
            .collect::<Vec<_>>();
        assert_eq!(entries.len(), 6);
        for index in 0..3 {
            let trace = format!("operation-{index}");
            let pair = entries
                .iter()
                .filter(|entry| entry["trace_id"] == trace)
                .collect::<Vec<_>>();
            assert_eq!(pair.len(), 2);
            assert_eq!(pair[0]["event"], "operation_started");
            assert_eq!(pair[1]["event"], "operation_completed");
        }
    }

    #[test]
    fn relative_logs_directory_does_not_depend_on_container_name() {
        let directory = tempdir().expect("temp directory");
        let runtime_root = directory.path().join("renamed-anywhere");
        fs::create_dir_all(&runtime_root).expect("create arbitrary runtime root");
        let path = create_session_log(
            &runtime_root,
            "cccccccc-0000-0000-0000-000000000000",
        )
        .expect("create relative runtime log");
        let logs_directory = runtime_root.join("logs");
        assert_eq!(path.parent(), Some(logs_directory.as_path()));
    }

    #[test]
    fn project_root_detection_skips_runtime_build_directory() {
        let directory = tempdir().expect("temp directory");
        let project_root = directory.path().join("football-platform");
        let source_root = project_root.join("project-source");
        let runtime_directory = project_root.join(".cargo-target").join("debug");
        fs::create_dir_all(&source_root).expect("create source root");
        fs::create_dir_all(&runtime_directory).expect("create runtime directory");
        fs::write(project_root.join("README.md"), "# project").expect("write README");
        fs::write(source_root.join("package.json"), "{}").expect("write package manifest");
        fs::write(source_root.join("Cargo.toml"), "[workspace]").expect("write cargo manifest");

        let candidates = runtime_root_candidates(&runtime_directory);
        assert_eq!(candidates.first(), Some(&project_root));
        assert!(!candidates.contains(&runtime_directory));
    }
}
