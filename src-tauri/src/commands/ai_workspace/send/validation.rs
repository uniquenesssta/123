use super::request::SendApiWorkspaceCommand;
use crate::commands::parse_uuid;
use football_application::{api_workspace_preset_spec, ApiWorkspacePresetSpec};
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct ValidatedMessage<'a> {
    pub(super) preset_spec: ApiWorkspacePresetSpec,
    pub(super) profile_id: &'a str,
    pub(super) message: &'a str,
    pub(super) request_id: &'a str,
    pub(super) match_id: Option<Uuid>,
    pub(super) command_entity_type: Option<String>,
    pub(super) command_entity_id: Option<Uuid>,
}

pub(super) fn validate(command: &SendApiWorkspaceCommand) -> Result<ValidatedMessage<'_>, String> {
    let preset_spec =
        api_workspace_preset_spec(command.preset_key.trim()).map_err(|error| error.to_string())?;
    let profile_id = command.profile_id.trim();
    if profile_id.is_empty() {
        return Err("请选择兼容 API 配置".to_string());
    }
    let message = command.message.trim();
    if message.is_empty() {
        return Err("请输入问题".to_string());
    }
    if !command.attachments.is_empty() {
        return Err("AI问答不支持附件；请使用 Excel 工作包维护资料".to_string());
    }
    let request_id = command.request_id.trim();
    if request_id.is_empty() || Uuid::parse_str(request_id).is_err() {
        return Err("AI问答请求 ID 无效".to_string());
    }
    let match_id = command
        .match_id
        .as_deref()
        .map(|value| parse_uuid(value, "比赛 ID"))
        .transpose()?;
    let command_entity_type = command
        .context_entity_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let command_entity_id = command
        .context_entity_id
        .as_deref()
        .map(|value| parse_uuid(value, "AI问答实体 ID"))
        .transpose()?;
    if command_entity_type.is_some() != command_entity_id.is_some() {
        return Err("AI问答实体上下文必须同时提供类型和 ID".to_string());
    }
    if command_entity_type
        .as_deref()
        .is_some_and(|value| !matches!(value, "team" | "player"))
    {
        return Err("AI问答实体上下文只支持 team 或 player".to_string());
    }

    Ok(ValidatedMessage {
        preset_spec,
        profile_id,
        message,
        request_id,
        match_id,
        command_entity_type,
        command_entity_id,
    })
}
