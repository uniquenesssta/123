use crate::{PersistenceError, PersistenceResult};
use football_domain::AiAnalysisResponsePreview;
use serde_json::Value;
use uuid::Uuid;

pub(super) fn validate_import_preview(
    preview: &AiAnalysisResponsePreview,
) -> PersistenceResult<()> {
    if !preview.blocking_errors.is_empty() {
        return Err(PersistenceError::InvalidState(format!(
            "AI 回包存在阻断错误：{}",
            preview.blocking_errors.join("；")
        )));
    }
    Ok(())
}

pub(super) fn suggestion_severity(severity: &str) -> &str {
    if severity.trim().is_empty() {
        "info"
    } else {
        severity
    }
}

pub(super) fn ensure_pending(status: &str) -> PersistenceResult<()> {
    if status != "pending" {
        return Err(PersistenceError::InvalidState(
            "该 AI 建议已处理".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
pub(super) struct AbilitySuggestionValues {
    pub(super) proposed_value: f64,
    pub(super) confidence: f64,
    pub(super) sample_size: i32,
}

pub(super) fn ability_values(payload: &Value) -> PersistenceResult<AbilitySuggestionValues> {
    let proposed_value = json_number(payload, "proposed_value")?.clamp(0.0, 100.0);
    let confidence = payload
        .get("confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.5)
        .clamp(0.0, 1.0);
    let sample_size = payload
        .get("sample_size")
        .and_then(Value::as_i64)
        .unwrap_or(0)
        .clamp(0, i32::MAX as i64) as i32;
    Ok(AbilitySuggestionValues {
        proposed_value,
        confidence,
        sample_size,
    })
}

pub(super) fn json_string(value: &Value, key: &str) -> PersistenceResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| PersistenceError::InvalidState(format!("AI 建议缺少 {key}")))
}

fn json_number(value: &Value, key: &str) -> PersistenceResult<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| PersistenceError::InvalidState(format!("AI 建议缺少数值 {key}")))
}

pub(super) fn json_uuid(value: &Value, key: &str) -> PersistenceResult<Uuid> {
    Uuid::parse_str(&json_string(value, key)?)
        .map_err(|error| PersistenceError::InvalidState(format!("AI 建议 {key} 无效：{error}")))
}
