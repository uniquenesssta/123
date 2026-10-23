use super::{
    metadata::{default_session_title, metadata_text, metadata_uuid},
    request::SendApiWorkspaceCommand,
    validation::ValidatedMessage,
};
use crate::{bootstrap::AppState, commands::parse_uuid, openai_profiles::OpenAiProfileSummary};
use football_domain::{ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord};
use serde_json::json;

pub(super) async fn resolve(
    state: &AppState,
    command: &SendApiWorkspaceCommand,
    input: &ValidatedMessage<'_>,
    profile: &OpenAiProfileSummary,
) -> Result<ApiWorkspaceSessionRecord, String> {
    let session = match command.session_id.as_deref() {
        Some(value) => {
            let session_id = parse_uuid(value, "AI问答会话 ID")?;
            let detail = state
                .service
                .read_api_workspace_session(session_id)
                .await
                .map_err(|error| error.to_string())?;
            if detail.session.status != "active" {
                return Err("该 AI 问答会话已归档".to_string());
            }
            if detail.session.profile_id != profile.id {
                return Err("当前会话绑定的 API 配置与所选配置不一致".to_string());
            }
            if detail.session.preset_key != input.preset_spec.preset.key {
                return Err("当前会话绑定的问答类型与所选类型不一致".to_string());
            }
            if detail.session.match_id != input.match_id {
                return Err("当前会话绑定的比赛与所选比赛不一致".to_string());
            }
            let stored_entity_type = metadata_text(&detail.session.metadata, "context_entity_type");
            let stored_entity_id = metadata_uuid(&detail.session.metadata, "context_entity_id")?;
            if stored_entity_type != input.command_entity_type
                || stored_entity_id != input.command_entity_id
            {
                return Err("当前会话绑定的球队或球员上下文与所选上下文不一致".to_string());
            }
            detail.session
        }
        None => {
            let title = command
                .title
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .unwrap_or_else(|| default_session_title(&input.preset_spec.preset, input.message));
            state
                .service
                .create_api_workspace_session(ApiWorkspaceSessionDraft {
                    profile_id: profile.id.clone(),
                    preset_key: input.preset_spec.preset.key.clone(),
                    title,
                    match_id: input.match_id,
                    metadata: json!({
                        "profile_name": profile.name.clone(),
                        "preset_title": input.preset_spec.preset.title.clone(),
                        "created_by": "desktop_ai_chat",
                        "context_entity_type": input.command_entity_type.clone(),
                        "context_entity_id": input.command_entity_id,
                        "context_entity_label": command.context_entity_label.as_deref().map(str::trim).filter(|value| !value.is_empty())
                    }),
                })
                .await
                .map_err(|error| error.to_string())?
        }
    };
    Ok(session)
}
