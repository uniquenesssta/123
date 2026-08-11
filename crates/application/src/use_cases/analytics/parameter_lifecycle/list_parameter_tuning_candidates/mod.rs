use crate::{ports::analytics::ParameterLifecyclePort, ApplicationResult};
use football_domain::ParameterTuningCandidateRecord;

pub(crate) async fn execute<P>(
    port: &P,
    limit: u32,
) -> ApplicationResult<Vec<ParameterTuningCandidateRecord>>
where
    P: ParameterLifecyclePort + ?Sized,
{
    Ok(port.list_tuning_candidates(limit).await?)
}
