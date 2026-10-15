use super::request::RuntimeOperationDraft;
use super::validation::*;
use serde_json::json;

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

#[test]
fn phases_keep_case_normalization_and_unknown_phase_error() {
    for phase in [
        "started",
        "completed",
        "failed",
        "ui_action",
        "navigation",
        "diagnostic",
    ] {
        assert_eq!(
            normalize_phase(&format!("  {}\n", phase.to_ascii_uppercase())).expect("phase"),
            phase
        );
    }
    assert_eq!(
        normalize_phase(" OTHER ").expect_err("unknown"),
        "不支持的运行日志阶段：other"
    );
    assert_eq!(
        normalize_phase("  ").expect_err("empty"),
        "不支持的运行日志阶段："
    );
}

#[test]
fn text_limits_count_unicode_scalars_and_preserve_exact_errors() {
    let operation = "球".repeat(MAX_OPERATION_NAME_CHARS);
    assert_eq!(
        validate_text(
            "操作名称",
            &format!(" {operation} "),
            MAX_OPERATION_NAME_CHARS
        )
        .expect("boundary"),
        operation
    );
    assert_eq!(
        validate_text("操作名称", " \n ", MAX_OPERATION_NAME_CHARS).expect_err("blank"),
        "操作名称不能为空"
    );
    assert_eq!(
        validate_text(
            "链路 ID",
            &"链".repeat(MAX_TRACE_ID_CHARS + 1),
            MAX_TRACE_ID_CHARS
        )
        .expect_err("too long"),
        "链路 ID过长：161 > 160"
    );
    assert_eq!(
        validate_text("链路 ID", " original-trace ", MAX_TRACE_ID_CHARS).expect("trace"),
        "original-trace"
    );
}

#[test]
fn runtime_draft_keeps_optional_nulls_unknown_fields_and_raw_input() {
    let draft: RuntimeOperationDraft = serde_json::from_value(json!({
        "phase": " COMPLETED ", "operation": " 原始操作 ", "ignored": true,
    }))
    .expect("original draft");
    assert_eq!(draft.phase, " COMPLETED ");
    assert_eq!(draft.operation, " 原始操作 ");
    assert!(draft.trace_id.is_none() && draft.duration_ms.is_none() && draft.context.is_none());
    let draft: RuntimeOperationDraft = serde_json::from_value(json!({
        "phase": "diagnostic", "operation": "x", "trace_id": "", "duration_ms": u64::MAX,
        "context": [1, "original"],
    }))
    .expect("full draft");
    assert_eq!(draft.trace_id.as_deref(), Some(""));
    assert_eq!(draft.duration_ms, Some(u64::MAX));
    assert_eq!(draft.context, Some(json!([1, "original"])));
}

#[test]
fn runtime_draft_keeps_required_fields_and_original_wire_types() {
    for draft in [
        json!({"operation": "x"}),
        json!({"phase": "started"}),
        json!({"phase": null, "operation": "x"}),
        json!({"phase": "started", "operation": "x", "duration_ms": -1}),
        json!({"phase": "started", "operation": "x", "trace_id": 1}),
    ] {
        assert!(serde_json::from_value::<RuntimeOperationDraft>(draft).is_err());
    }
}
