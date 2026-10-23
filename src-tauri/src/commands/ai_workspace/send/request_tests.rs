use super::request::SendApiWorkspaceCommand;
use serde_json::json;

#[test]
fn request_keeps_wire_defaults_unknown_fields_and_raw_strings() {
    let command: SendApiWorkspaceCommand = serde_json::from_value(json!({
        "profile_id": " profile ", "preset_key": " plain_chat ", "message": " 问题 ",
        "future_field": true
    }))
    .unwrap();
    assert_eq!(command.profile_id, " profile ");
    assert_eq!(command.message, " 问题 ");
    assert_eq!(command.preset_key, " plain_chat ");
    assert_eq!(command.session_id, None);
    assert_eq!(command.title, None);
    assert_eq!(command.match_id, None);
    assert_eq!(command.context_entity_type, None);
    assert_eq!(command.context_entity_id, None);
    assert_eq!(command.context_entity_label, None);
    assert!(!command.include_context);
    assert!(command.request_id.is_empty());
    assert!(command.attachments.is_empty());
}

#[test]
fn request_rejects_missing_required_fields_and_wrong_wire_types() {
    let valid = json!({"profile_id":"p","preset_key":"plain_chat","message":"q"});
    for required in ["profile_id", "preset_key", "message"] {
        let mut input = valid.clone();
        input.as_object_mut().unwrap().remove(required);
        assert!(serde_json::from_value::<SendApiWorkspaceCommand>(input).is_err());
    }
    for (key, value) in [
        ("include_context", json!("true")),
        ("attachments", json!({})),
        ("request_id", json!(null)),
        ("match_id", json!(7)),
    ] {
        let mut input = valid.clone();
        input[key] = value;
        assert!(serde_json::from_value::<SendApiWorkspaceCommand>(input).is_err());
    }
}
