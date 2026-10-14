use crate::{ports::analytics::ParameterLifecyclePort, ApplicationError, ApplicationResult};
use football_domain::{
    ParameterTuningCandidateRecord, ParameterTuningDecisionDraft, ParameterTuningDraft,
};

pub(crate) async fn generate_parameter_tuning_candidate(
    draft: ParameterTuningDraft,
) -> ApplicationResult<ParameterTuningCandidateRecord> {
    let _ = draft;
    Err(ApplicationError::Model(
        "公开仓库未捆绑模型提供器，不能生成提供器私有参数候选".to_string(),
    ))
}

pub(crate) async fn list_parameter_tuning_candidates<P>(
    port: &P,
    limit: u32,
) -> ApplicationResult<Vec<ParameterTuningCandidateRecord>>
where
    P: ParameterLifecyclePort + ?Sized,
{
    Ok(port.list_tuning_candidates(limit).await?)
}

pub(crate) async fn decide_parameter_tuning_candidate<P>(
    port: &P,
    draft: ParameterTuningDecisionDraft,
) -> ApplicationResult<ParameterTuningCandidateRecord>
where
    P: ParameterLifecyclePort + ?Sized,
{
    Ok(port.decide_tuning_candidate(&draft).await?)
}
