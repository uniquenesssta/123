use crate::{ports::analytics::ParameterLifecyclePort, ApplicationError, ApplicationResult};
use football_domain::{ParameterPromotionDecisionRecord, ParameterRollbackRequest};

pub(crate) async fn execute<P>(
    port: &P,
    request: ParameterRollbackRequest,
) -> ApplicationResult<ParameterPromotionDecisionRecord>
where
    P: ParameterLifecyclePort + ?Sized,
{
    if request.decision_note.trim().len() < 8 {
        return Err(ApplicationError::Validation(
            "回滚说明至少需要 8 个字符".to_string(),
        ));
    }
    Ok(port.rollback(&request).await?)
}
