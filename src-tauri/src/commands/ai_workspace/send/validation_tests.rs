use super::{metadata, request::SendApiWorkspaceCommand, validation::validate};
use serde_json::json;
use uuid::Uuid;

fn command() -> SendApiWorkspaceCommand {
    serde_json::from_value(json!({
        "profile_id":" p ","preset_key":" plain_chat ","message":" 问题 ",
        "request_id":" 00000000-0000-0000-0000-000000000001 "
    }))
    .unwrap()
}

#[test]
fn validation_preserves_error_priority_before_registration_or_io() {
    let mut input = command();
    input.profile_id = " ".into();
    input.message = " ".into();
    input.request_id.clear();
    input.attachments = serde_json::from_value(json!([{
        "name":"x","media_type":"text/plain","content":"x","content_sha256":"x",
        "original_size_bytes":1,"truncated":false
    }]))
    .unwrap();
    assert_eq!(validate(&input).unwrap_err(), "请选择兼容 API 配置");
    input.profile_id = "p".into();
    assert_eq!(validate(&input).unwrap_err(), "请输入问题");
    input.message = "q".into();
    assert_eq!(
        validate(&input).unwrap_err(),
        "AI问答不支持附件；请使用 Excel 工作包维护资料"
    );
    input.attachments.clear();
    assert_eq!(validate(&input).unwrap_err(), "AI问答请求 ID 无效");
}

#[test]
fn validation_keeps_normalization_and_rejects_invalid_optional_identity() {
    let mut input = command();
    let result = validate(&input).unwrap();
    assert_eq!(result.profile_id, "p");
    assert_eq!(result.message, "问题");
    assert_eq!(result.request_id, "00000000-0000-0000-0000-000000000001");
    assert_eq!(result.preset_spec.preset.key, "plain_chat");
    input.match_id = Some(" invalid ".into());
    assert!(validate(&input).unwrap_err().starts_with("比赛 ID 无效："));
    input.match_id = None;
    input.context_entity_type = Some(" team ".into());
    assert_eq!(
        validate(&input).unwrap_err(),
        "AI问答实体上下文必须同时提供类型和 ID"
    );
    input.context_entity_id = Some(Uuid::nil().to_string());
    assert_eq!(
        validate(&input).unwrap().command_entity_type.as_deref(),
        Some("team")
    );
    input.context_entity_type = Some("coach".into());
    assert_eq!(
        validate(&input).unwrap_err(),
        "AI问答实体上下文只支持 team 或 player"
    );
}

#[test]
fn session_metadata_keeps_absence_trim_and_invalid_uuid_errors() {
    assert_eq!(metadata::metadata_uuid(&json!({}), "id").unwrap(), None);
    assert_eq!(metadata::metadata_text(&json!({"id":"  "}), "id"), None);
    assert_eq!(
        metadata::metadata_uuid(&json!({"id":Uuid::nil().to_string()}), "id").unwrap(),
        Some(Uuid::nil())
    );
    assert!(metadata::metadata_uuid(&json!({"id":"invalid"}), "id")
        .unwrap_err()
        .starts_with("AI问答会话实体 ID 无效："));
}

#[test]
fn session_title_collapses_whitespace_and_limits_unicode_characters() {
    let preset = football_application::api_workspace_presets().remove(0);
    assert_eq!(
        metadata::default_session_title(&preset, " \n\t "),
        preset.title
    );
    assert_eq!(
        metadata::default_session_title(&preset, " 一\n二\t三 "),
        format!("{} · 一 二 三", preset.title)
    );
    assert_eq!(
        metadata::default_session_title(&preset, &"球".repeat(50)),
        format!("{} · {}", preset.title, "球".repeat(42))
    );
}
