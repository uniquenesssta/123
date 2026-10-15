use super::{
    request::RuntimeOperationDraft,
    validation::{normalize_phase, validate_text, MAX_OPERATION_NAME_CHARS, MAX_TRACE_ID_CHARS},
};
use crate::bootstrap::AppState;
use serde_json::{json, Value};
use tauri::State;

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
