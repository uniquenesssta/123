use super::*;

pub(super) fn verification_priority(state: EvidenceVerificationState) -> u8 {
    match state {
        EvidenceVerificationState::Confirmed => 6,
        EvidenceVerificationState::Probable => 5,
        EvidenceVerificationState::Conflict => 4,
        EvidenceVerificationState::Stale => 3,
        EvidenceVerificationState::NotFound => 2,
        EvidenceVerificationState::NotApplicable => 1,
    }
}

pub(super) fn parse_verification_state(
    value: &str,
) -> ApplicationResult<EvidenceVerificationState> {
    match value {
        "CONFIRMED" => Ok(EvidenceVerificationState::Confirmed),
        "PROBABLE" => Ok(EvidenceVerificationState::Probable),
        "CONFLICT" => Ok(EvidenceVerificationState::Conflict),
        "NOT_FOUND" => Ok(EvidenceVerificationState::NotFound),
        "STALE" => Ok(EvidenceVerificationState::Stale),
        "NOT_APPLICABLE" => Ok(EvidenceVerificationState::NotApplicable),
        other => Err(ApplicationError::Validation(format!(
            "未知证据状态：{other}"
        ))),
    }
}

pub(super) fn canonical_json(value: &Value) -> ApplicationResult<String> {
    Ok(serde_json::to_string(value)?)
}

pub(super) fn sha256_text(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

pub(super) fn validate_pipeline_command(
    command: &ProcessResearchEvidenceCommand,
) -> ApplicationResult<()> {
    if command.response_id.trim().is_empty() || command.response_id.chars().count() > 200 {
        return Err(ApplicationError::Validation(
            "事实流水线必须关联有效response_id".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn validate_pipeline_context(
    command: &ProcessResearchEvidenceCommand,
    context: &FactPipelineContext,
) -> ApplicationResult<()> {
    if command.output.match_key != context.match_key
        || command.output.data_cutoff_at != context.data_cutoff_at
        || command.output.schema_version != context.schema_version
    {
        return Err(ApplicationError::Validation(
            "联网结果与研究任务的比赛键、截止时间或Schema版本不一致".to_string(),
        ));
    }
    Ok(())
}
