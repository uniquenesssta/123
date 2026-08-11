use crate::{ports::analytics::ParameterLifecyclePort, ApplicationResult};
use football_domain::{ParameterTuningCandidateRecord, ParameterTuningDecisionDraft};

pub(crate) async fn execute<P>(
    port: &P,
    draft: ParameterTuningDecisionDraft,
) -> ApplicationResult<ParameterTuningCandidateRecord>
where
    P: ParameterLifecyclePort + ?Sized,
{
    Ok(port.decide_tuning_candidate(&draft).await?)
}
