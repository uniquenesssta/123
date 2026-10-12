use crate::{PersistenceError, PersistenceResult};
use football_domain::{ApiWorkspaceGeneratedFileDraft, ApiWorkspaceMessageDraft};

pub(super) fn validate_message_draft(draft: &ApiWorkspaceMessageDraft) -> PersistenceResult<()> {
    if !matches!(draft.role.as_str(), "user" | "assistant" | "system") {
        return Err(PersistenceError::InvalidState(
            "API协作消息角色无效".to_string(),
        ));
    }
    if draft.content.chars().count() > 100_000 {
        return Err(PersistenceError::InvalidState(
            "API协作消息超过100000字符".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_generated_file_draft(
    draft: &ApiWorkspaceGeneratedFileDraft,
) -> PersistenceResult<()> {
    if draft.filename.trim().is_empty()
        || draft.filename.chars().count() > 120
        || draft.filename.chars().any(|ch| "\\/:*?\"<>|".contains(ch))
        || !matches!(
            draft.media_type.as_str(),
            "text/plain" | "text/markdown" | "application/json" | "text/csv"
        )
        || draft.content.len() > 2 * 1024 * 1024
    {
        return Err(PersistenceError::InvalidState(
            "API生成文件名称、类型或大小无效".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;
    fn message(role: &str, content: String) -> ApiWorkspaceMessageDraft {
        ApiWorkspaceMessageDraft {
            session_id: Uuid::nil(),
            role: role.into(),
            content,
            structured_payload: json!({}),
            citations: json!([]),
            attachments: json!([]),
            provider_response_id: None,
            model_id: None,
            token_usage: json!({}),
        }
    }
    fn file() -> ApiWorkspaceGeneratedFileDraft {
        ApiWorkspaceGeneratedFileDraft {
            session_id: Uuid::nil(),
            message_id: Uuid::nil(),
            filename: "file.txt".into(),
            media_type: "text/plain".into(),
            content: "".into(),
            content_sha256: "".into(),
        }
    }
    #[test]
    fn message_roles_keep_exact_values_and_empty_content_compatibility() {
        for role in ["user", "assistant", "system"] {
            validate_message_draft(&message(role, String::new())).unwrap();
        }
        for role in [" user ", "tool", "USER", ""] {
            let error = validate_message_draft(&message(role, "a".repeat(100_001))).unwrap_err();
            assert!(
                matches!(error,PersistenceError::InvalidState(message) if message=="API协作消息角色无效")
            );
        }
    }
    #[test]
    fn message_limit_counts_unicode_characters_and_keeps_original_error() {
        validate_message_draft(&message("user", "足".repeat(100_000))).unwrap();
        let error = validate_message_draft(&message("user", "足".repeat(100_001))).unwrap_err();
        assert!(
            matches!(error,PersistenceError::InvalidState(message) if message=="API协作消息超过100000字符")
        );
    }
    #[test]
    fn file_policy_preserves_name_media_type_and_raw_whitespace_rules() {
        for media in [
            "text/plain",
            "text/markdown",
            "application/json",
            "text/csv",
        ] {
            let mut value = file();
            value.media_type = media.into();
            value.filename = " 名称.txt ".into();
            validate_generated_file_draft(&value).unwrap();
        }
        for name in [
            "", " \n ", "a/b", "a\\b", "a:b", "a*b", "a?b", "a\"b", "a<b", "a>b", "a|b",
        ] {
            let mut value = file();
            value.filename = name.into();
            assert!(
                matches!(validate_generated_file_draft(&value),Err(PersistenceError::InvalidState(message)) if message=="API生成文件名称、类型或大小无效")
            );
        }
        for media in ["text/plain ", "TEXT/PLAIN", "application/octet-stream"] {
            let mut value = file();
            value.media_type = media.into();
            assert!(validate_generated_file_draft(&value).is_err());
        }
    }
    #[test]
    fn file_limits_preserve_character_names_byte_payloads_and_supplied_hash() {
        let mut value = file();
        value.filename = "足".repeat(120);
        value.content = "a".repeat(2 * 1024 * 1024);
        validate_generated_file_draft(&value).unwrap();
        value.filename.push('足');
        assert!(validate_generated_file_draft(&value).is_err());
        value.filename = "file.txt".into();
        value.content.push('a');
        assert!(validate_generated_file_draft(&value).is_err());
        value.content = "足".repeat(700_000);
        assert!(validate_generated_file_draft(&value).is_err());
        value.content = "原样".into();
        validate_generated_file_draft(&value).unwrap();
        assert_eq!(value.content_sha256, "");
    }
}
