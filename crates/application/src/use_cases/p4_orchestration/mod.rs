mod failure;
pub(crate) mod process_next;

use crate::ports::{
    analytics::JobQueuePort,
    prediction::P4OrchestrationQueuePort,
    research::{ResearchArtifactPort, ResearchGatewayAuditPort},
};
use crate::use_cases::{
    prediction::P4FreezeExecutionAccess, research::fact_pipeline::FactPipelineAccess,
};

pub(crate) trait P4OrchestrationAccess:
    P4OrchestrationQueuePort
    + P4FreezeExecutionAccess
    + JobQueuePort
    + ResearchArtifactPort
    + ResearchGatewayAuditPort
    + FactPipelineAccess
{
}

impl<T> P4OrchestrationAccess for T where
    T: P4OrchestrationQueuePort
        + P4FreezeExecutionAccess
        + JobQueuePort
        + ResearchArtifactPort
        + ResearchGatewayAuditPort
        + FactPipelineAccess
        + ?Sized
{
}
