use crate::{ports::analytics::ParameterLifecyclePort, ApplicationResult};
use football_domain::ParameterShadowValidationRecord;
use uuid::Uuid;

pub(crate) async fn execute<P>(
    port: &P,
    candidate_id: Uuid,
) -> ApplicationResult<Vec<ParameterShadowValidationRecord>>
where
    P: ParameterLifecyclePort + ?Sized,
{
    Ok(port.list_shadow_validations(candidate_id).await?)
}
