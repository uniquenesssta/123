use super::metrics::{minimum_sample_size, required_candidate_uuid};
use crate::{ports::analytics::ParameterLifecyclePort, ApplicationError, ApplicationResult};
use football_domain::{
    ParameterLifecycleReadinessRequest, ParameterPromotionDecisionRecord, ParameterPromotionRequest,
};

pub(crate) async fn execute<P>(
    port: &P,
    request: ParameterPromotionRequest,
) -> ApplicationResult<ParameterPromotionDecisionRecord>
where
    P: ParameterLifecyclePort + ?Sized,
{
    if request.decision_note.trim().len() < 8 {
        return Err(ApplicationError::Validation(
            "人工晋升说明至少需要 8 个字符".to_string(),
        ));
    }
    let candidate = port.read_tuning_candidate(request.candidate_id).await?;
    let competition_id = required_candidate_uuid(candidate.competition_id, "赛事范围")?;
    let readiness = port
        .readiness(&ParameterLifecycleReadinessRequest {
            competition_id: Some(competition_id),
            snapshot_type: candidate.snapshot_type.clone(),
            minimum_sample_size: minimum_sample_size(&candidate),
        })
        .await?;
    if !readiness.ready_for_promotion {
        return Err(ApplicationError::Validation(format!(
            "晋升门禁未通过：{}",
            readiness.blocked_reasons.join("；")
        )));
    }
    if readiness.active_model_version_id != candidate.baseline_model_version_id
        || readiness.active_parameter_set_id != candidate.baseline_parameter_set_id
    {
        return Err(ApplicationError::Validation(
            "正式绑定已发生变化，候选必须重新校准和影子验证".to_string(),
        ));
    }
    Ok(port.promote(&request).await?)
}
