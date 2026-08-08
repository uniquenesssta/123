use crate::{ApplicationError, ApplicationResult};
use football_domain::RulePackageDraft;
use serde_json::Value;

pub(crate) fn validate_rule_package_shape(draft: &RulePackageDraft) -> ApplicationResult<()> {
    for (label, value) in [
        ("format_version", &draft.format_version),
        ("package_key", &draft.package_key),
        ("version", &draft.version),
        ("display_name", &draft.display_name),
        ("profile_id", &draft.competition_profile.profile_id),
        ("model_id", &draft.routing.model_id),
        ("model_version", &draft.routing.model_version),
        ("parameter_version", &draft.routing.parameter_version),
    ] {
        if value.trim().is_empty() {
            return Err(ApplicationError::Validation(format!(
                "规则包字段 {label} 不能为空"
            )));
        }
    }
    if draft.format_version != "football.rule-package.v1" {
        return Err(ApplicationError::Validation(format!(
            "不支持的规则包格式：{}",
            draft.format_version
        )));
    }
    if draft.competition_profile.normal_time_minutes == 0 {
        return Err(ApplicationError::Validation(
            "正常比赛时间必须大于 0".to_string(),
        ));
    }
    if !draft.parameters.is_object() {
        return Err(ApplicationError::Validation(
            "规则包 parameters 必须是 JSON 对象".to_string(),
        ));
    }
    if !draft.feature_requirements.is_object() || !draft.output_contract.is_object() {
        return Err(ApplicationError::Validation(
            "feature_requirements 和 output_contract 必须是 JSON 对象".to_string(),
        ));
    }
    if let Some(source) = &draft.source_document {
        let hash = source
            .content_sha256
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                ApplicationError::Validation("规则包包含来源文档时必须提供文档 SHA-256".to_string())
            })?;
        if hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ApplicationError::Validation(
                "来源文档 SHA-256 必须是 64 位十六进制字符串".to_string(),
            ));
        }
    }

    let parameter_profile = draft
        .parameters
        .get("profile")
        .and_then(Value::as_object)
        .ok_or_else(|| ApplicationError::Validation("parameters.profile 缺失".to_string()))?;
    let parameter_profile_id = parameter_profile
        .get("profile_id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if parameter_profile_id != draft.competition_profile.profile_id {
        return Err(ApplicationError::Validation(format!(
            "parameters.profile.profile_id 与 competition_profile.profile_id 不一致：{} != {}",
            parameter_profile_id, draft.competition_profile.profile_id
        )));
    }
    let parameter_kind = parameter_profile
        .get("competition_type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if parameter_kind != draft.competition_profile.competition_kind.as_str() {
        return Err(ApplicationError::Validation(format!(
            "parameters.profile.competition_type 与规则包赛事类型不一致：{} != {}",
            parameter_kind,
            draft.competition_profile.competition_kind.as_str()
        )));
    }
    Ok(())
}

pub(crate) fn validate_parameter_identity(draft: &RulePackageDraft) -> ApplicationResult<()> {
    let parameter_model_version = required_string(&draft.parameters, "model_version")?;
    let parameter_version = required_string(&draft.parameters, "parameter_version")?;
    if parameter_model_version != draft.routing.model_version {
        return Err(ApplicationError::Validation(format!(
            "parameters.model_version 与 routing.model_version 不一致：{} != {}",
            parameter_model_version, draft.routing.model_version
        )));
    }
    if parameter_version != draft.routing.parameter_version {
        return Err(ApplicationError::Validation(format!(
            "parameters.parameter_version 与 routing.parameter_version 不一致：{} != {}",
            parameter_version, draft.routing.parameter_version
        )));
    }
    Ok(())
}

fn required_string(value: &Value, key: &str) -> ApplicationResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| ApplicationError::Model(format!("缺少字符串字段：{key}")))
}
