use crate::file_store::{remove_if_exists, write_atomic};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

const MAX_WORKSPACE_STATE_BYTES: usize = 1024 * 1024;
const SCHEMA_VERSION: u64 = 1;
const FORBIDDEN_KEY_PARTS: &[&str] = &[
    "api_key",
    "apikey",
    "password",
    "secret",
    "credential",
    "attachment_body",
    "file_content",
    "database_url",
];

pub struct WorkspaceStateStore {
    path: PathBuf,
}

impl WorkspaceStateStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn read(&self) -> Result<Value, String> {
        match fs::read(&self.path) {
            Ok(bytes) => {
                if bytes.len() > MAX_WORKSPACE_STATE_BYTES {
                    return Err("工作区状态文件超过 1 MiB 安全上限".to_string());
                }
                let value: Value = serde_json::from_slice(&bytes)
                    .map_err(|error| format!("工作区状态文件格式无效：{error}"))?;
                validate_document(&value)?;
                Ok(value)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(empty_document()),
            Err(error) => Err(format!(
                "无法读取工作区状态文件 {}：{error}",
                self.path.display()
            )),
        }
    }

    pub fn write(&self, value: &Value) -> Result<(), String> {
        validate_document(value)?;
        let bytes = serde_json::to_vec_pretty(value)
            .map_err(|error| format!("无法序列化工作区状态：{error}"))?;
        if bytes.len() > MAX_WORKSPACE_STATE_BYTES {
            return Err("工作区状态超过 1 MiB 安全上限".to_string());
        }
        write_atomic(&self.path, &bytes, true)
    }

    pub fn clear(&self) -> Result<Value, String> {
        remove_if_exists(&self.path)?;
        Ok(empty_document())
    }
}

fn empty_document() -> Value {
    json!({ "schema_version": SCHEMA_VERSION, "global": {}, "modules": {} })
}

fn validate_document(value: &Value) -> Result<(), String> {
    let root = value
        .as_object()
        .ok_or_else(|| "工作区状态必须是 JSON 对象".to_string())?;
    if root.get("schema_version").and_then(Value::as_u64) != Some(SCHEMA_VERSION) {
        return Err(format!("工作区状态 Schema 必须为 {SCHEMA_VERSION}"));
    }
    if !root.get("global").is_some_and(Value::is_object) {
        return Err("工作区状态缺少 global 对象".to_string());
    }
    if !root.get("modules").is_some_and(Value::is_object) {
        return Err("工作区状态缺少 modules 对象".to_string());
    }
    reject_sensitive_keys(value, "root")
}

fn reject_sensitive_keys(value: &Value, path: &str) -> Result<(), String> {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let normalized = key.to_ascii_lowercase();
                if FORBIDDEN_KEY_PARTS
                    .iter()
                    .any(|part| normalized.contains(part))
                {
                    return Err(format!("工作区状态包含禁止持久化字段：{path}.{key}"));
                }
                reject_sensitive_keys(child, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                reject_sensitive_keys(child, &format!("{path}[{index}]"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_state_round_trip_and_clear_are_safe() {
        let directory = tempfile::tempdir().expect("创建临时目录");
        let path = directory.path().join("workspace-state.json");
        let store = WorkspaceStateStore::new(path.clone());
        let value = json!({
            "schema_version": 1,
            "global": {"sidebar_collapsed": true},
            "modules": {"teams": {"active_tab_id": "team-1"}}
        });
        store.write(&value).expect("写入状态");
        assert_eq!(store.read().expect("读取状态"), value);
        assert_eq!(store.clear().expect("清空状态"), empty_document());
        assert!(!path.exists());
    }

    #[test]
    fn workspace_state_rejects_sensitive_keys() {
        let value = json!({
            "schema_version": 1,
            "global": {},
            "modules": {"openai": {"api_key": "secret"}}
        });
        assert!(validate_document(&value).is_err());
    }
}
