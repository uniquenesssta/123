use super::AppState;
use serde::Deserialize;
use serde_json::{json, Value};
use tauri::State;

const MAX_OPERATION_NAME_CHARS: usize = 160;
const MAX_TRACE_ID_CHARS: usize = 160;

#[derive(Debug, Deserialize)]
pub struct RuntimeOperationDraft {
    phase: String,
    operation: String,
    trace_id: Option<String>,
    duration_ms: Option<u64>,
    context: Option<Value>,
}

#[tauri::command]
pub fn record_runtime_operation(
    state: State<'_, AppState>,
    draft: RuntimeOperationDraft,
) -> Result<(), String> {
    let phase = normalize_phase(&draft.phase)?;
    let operation = validate_text("操作名称", &draft.operation, MAX_OPERATION_NAME_CHARS)?;
    let trace_id = draft
        .trace_id
        .as_deref()
        .map(|value| validate_text("链路 ID", value, MAX_TRACE_ID_CHARS))
        .transpose()?;
    let level = match phase {
        "failed" => "error",
        _ => "info",
    };
    state.runtime_log.record(
        level,
        "frontend.operation",
        &format!("operation_{phase}"),
        trace_id.as_deref(),
        json!({
            "operation": operation,
            "duration_ms": draft.duration_ms,
            "context": draft.context.unwrap_or(Value::Null),
        }),
    )
}

fn normalize_phase(value: &str) -> Result<&'static str, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "started" => Ok("started"),
        "completed" => Ok("completed"),
        "failed" => Ok("failed"),
        "ui_action" => Ok("ui_action"),
        "navigation" => Ok("navigation"),
        "diagnostic" => Ok("diagnostic"),
        other => Err(format!("不支持的运行日志阶段：{other}")),
    }
}

fn validate_text(label: &str, value: &str, max_chars: usize) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("{label}不能为空"));
    }
    let char_count = trimmed.chars().count();
    if char_count > max_chars {
        return Err(format!("{label}过长：{char_count} > {max_chars}"));
    }
    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_validation_rejects_unknown_values() {
        assert!(normalize_phase("unknown").is_err());
        assert_eq!(normalize_phase("completed").expect("phase"), "completed");
        assert_eq!(normalize_phase("diagnostic").expect("phase"), "diagnostic");
    }

    #[test]
    fn operation_name_validation_is_bounded() {
        assert!(validate_text("操作名称", "", MAX_OPERATION_NAME_CHARS).is_err());
        assert!(validate_text(
            "操作名称",
            &"x".repeat(MAX_OPERATION_NAME_CHARS + 1),
            MAX_OPERATION_NAME_CHARS,
        )
        .is_err());
    }
}
