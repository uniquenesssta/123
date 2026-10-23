use crate::commands::parse_uuid;
use football_domain::ApiWorkspacePreset;
use serde_json::Value;
use uuid::Uuid;

pub(super) fn metadata_text(metadata: &Value, key: &str) -> Option<String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn metadata_uuid(metadata: &Value, key: &str) -> Result<Option<Uuid>, String> {
    metadata_text(metadata, key)
        .as_deref()
        .map(|value| parse_uuid(value, "AI问答会话实体 ID"))
        .transpose()
}

pub(super) fn default_session_title(preset: &ApiWorkspacePreset, message: &str) -> String {
    let compact = message.split_whitespace().collect::<Vec<_>>().join(" ");
    let excerpt = compact.chars().take(42).collect::<String>();
    if excerpt.is_empty() {
        preset.title.clone()
    } else {
        format!("{} · {}", preset.title, excerpt)
    }
}
