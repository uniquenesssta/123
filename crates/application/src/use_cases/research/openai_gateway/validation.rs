use super::*;

pub(super) fn validate_command(command: &OpenAiResearchCommand) -> ApplicationResult<()> {
    if command.match_key.trim().is_empty()
        || command.match_key.chars().count() > 200
        || command.requested_fact_keys.is_empty()
        || command.requested_fact_keys.len() > 31
    {
        return Err(ApplicationError::Validation(
            "OpenAI研究任务必须包含有效比赛键和1至31个事实字段".to_string(),
        ));
    }
    if command
        .requested_fact_keys
        .iter()
        .any(|field| field.trim().is_empty() || field.chars().count() > 100)
    {
        return Err(ApplicationError::Validation(
            "OpenAI研究任务包含空字段或超过100字符的字段".to_string(),
        ));
    }
    let unique_fields: std::collections::BTreeSet<_> = command
        .requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect();
    if unique_fields.len() != command.requested_fact_keys.len() {
        return Err(ApplicationError::Validation(
            "OpenAI研究任务不能包含重复事实字段".to_string(),
        ));
    }
    Ok(())
}
