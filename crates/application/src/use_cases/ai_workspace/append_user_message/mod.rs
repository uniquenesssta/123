use crate::{ports::ai_workspace::ApiWorkspaceSessionPort, ApplicationError, ApplicationResult};
use football_domain::{
    ApiWorkspaceAttachment, ApiWorkspaceMessageDraft, ApiWorkspaceMessageRecord,
};
use serde_json::{json, Value};
use std::future::Future;
use uuid::Uuid;

pub(crate) async fn execute<P, F>(
    session: F,
    session_id: Uuid,
    content: String,
    attachments: &[ApiWorkspaceAttachment],
) -> ApplicationResult<ApiWorkspaceMessageRecord>
where
    P: ApiWorkspaceSessionPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let trimmed = content.trim();
    if trimmed.is_empty() && attachments.is_empty() {
        return Err(ApplicationError::Validation(
            "请输入问题或选择附件".to_string(),
        ));
    }
    let attachment_metadata = attachments
        .iter()
        .map(|attachment| {
            json!({
                "name": attachment.name,
                "media_type": attachment.media_type,
                "content_sha256": attachment.content_sha256,
                "original_size_bytes": attachment.original_size_bytes,
                "truncated": attachment.truncated
            })
        })
        .collect::<Vec<_>>();
    let draft = ApiWorkspaceMessageDraft {
        session_id,
        role: "user".to_string(),
        content: trimmed.to_string(),
        structured_payload: json!({}),
        citations: json!([]),
        attachments: Value::Array(attachment_metadata),
        provider_response_id: None,
        model_id: None,
        token_usage: json!({}),
    };
    Ok(session.await?.append_message(&draft).await?)
}
