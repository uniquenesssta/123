use crate::{ports::analytics::ParameterLifecyclePort, ApplicationResult};
use football_domain::ParameterPromotionDecisionRecord;
use uuid::Uuid;

pub(crate) async fn execute<P>(
    port: &P,
    candidate_id: Uuid,
) -> ApplicationResult<Vec<ParameterPromotionDecisionRecord>>
where
    P: ParameterLifecyclePort + ?Sized,
{
    Ok(port.list_promotion_decisions(candidate_id).await?)
}
