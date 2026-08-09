use crate::built_in_artifacts::{
    P4_RESEARCH_PROMPT_ARTIFACT_VERSION, P4_RESEARCH_PROMPT_KEY as RESEARCH_PROMPT_KEY,
    P4_RESEARCH_SCHEMA_ARTIFACT_VERSION, P4_RESEARCH_SCHEMA_KEY as RESEARCH_SCHEMA_KEY,
};
use crate::ports::research::{ResearchArtifactPort, ResearchGatewayAuditPort};
use crate::use_cases::research::fact_pipeline::{
    self, FactPipelineAccess, ProcessResearchEvidenceCommand,
};
use crate::{ApplicationError, ApplicationResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{
    OpenAiAttemptDraft, PromptVersionDraft, ResearchRunEventDraft, ResearchRunStatus,
    SchemaVersionDraft, WebCitationDraft, WebSourceDraft, P4_RESEARCH_OUTPUT_SCHEMA_VERSION,
    P4_RESEARCH_PROMPT_VERSION,
};
use football_research_gateway::{
    CancellationToken, DefaultApiKeyProvider, GatewayAttempt, GatewayAttemptSink, GatewayConfig,
    GatewayError, GatewayErrorCategory, GatewayExecution, GatewayOperation, GatewayRequest,
    OpenAiResearchGateway, ReqwestTransport, WebCitation, WebSource,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

mod artifacts;
mod attempt_audit;
mod execution;
mod gateway;
mod references;
mod types;
mod validation;

pub use types::OpenAiResearchCommand;

const RESEARCH_SCHEMA_NAME: &str = "p4_openai_research_output";

pub(crate) async fn register_openai_research_artifacts(
    port: &dyn ResearchArtifactPort,
) -> ApplicationResult<()> {
    artifacts::register(port).await
}

pub(crate) async fn execute_p4_openai_research(
    artifacts: &dyn ResearchArtifactPort,
    audit: &dyn ResearchGatewayAuditPort,
    pipeline: &dyn FactPipelineAccess,
    command: OpenAiResearchCommand,
    cancellation: CancellationToken,
) -> ApplicationResult<GatewayExecution> {
    execution::execute(artifacts, audit, pipeline, command, cancellation).await
}

#[cfg(test)]
mod tests;
