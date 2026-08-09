from pathlib import Path
import re
import textwrap

ROOT = Path(__file__).resolve().parents[1]
LEGACY = ROOT / "crates/application/src/openai_research.rs"
source = LEGACY.read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content.rstrip() + "\n", encoding="utf-8", newline="\n")


def braced(text: str, marker: str, start: int = 0) -> str:
    pos = text.find(marker, start)
    if pos < 0:
        raise RuntimeError(f"missing marker: {marker}")
    open_brace = text.find("{", pos)
    if open_brace < 0:
        raise RuntimeError(f"missing opening brace: {marker}")
    depth = 0
    in_string = False
    escape = False
    i = open_brace
    while i < len(text):
        ch = text[i]
        if in_string:
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == '"':
                in_string = False
        else:
            if ch == '"':
                in_string = True
            elif ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    return text[pos:i + 1]
        i += 1
    raise RuntimeError(f"unclosed braces: {marker}")


def fn(name: str) -> str:
    match = re.search(rf"(?m)^(?:async\s+)?fn\s+{re.escape(name)}\s*\(", source)
    if not match:
        raise RuntimeError(f"missing function: {name}")
    return braced(source, match.group(0), match.start())


def struct_with_attrs(name: str) -> str:
    marker = f"pub struct {name}"
    pos = source.find(marker)
    if pos < 0:
        raise RuntimeError(f"missing struct: {name}")
    start = source.rfind("#[derive", 0, pos)
    if start < 0:
        start = pos
    block = braced(source, marker, pos)
    end = source.find("}", pos)
    return source[start:pos] + block


def test_body() -> str:
    block = braced(source, "mod tests {")
    open_brace = block.find("{")
    return textwrap.dedent(block[open_brace + 1:-1]).strip()


command = struct_with_attrs("OpenAiResearchCommand")
default_op = fn("default_operation")
write(
    "crates/application/src/use_cases/research/openai_gateway/types.rs",
    "use super::*;\n\n" + command + "\n\n" + default_op,
)

artifacts_functions = "\n\n".join([fn("built_in_research_schema"), fn("built_in_research_prompt")])
artifacts_functions = artifacts_functions.replace("../../../src-tauri", "../../../../../../src-tauri")
write(
    "crates/application/src/use_cases/research/openai_gateway/artifacts.rs",
    "use super::*;\n\n"
    "pub(super) async fn register(port: &dyn ResearchArtifactPort) -> ApplicationResult<()> {\n"
    "    port.register_schema(&built_in_research_schema()).await?;\n"
    "    port.register_prompt(&built_in_research_prompt()).await?;\n"
    "    fact_pipeline::register_fact_pipeline_artifacts(port).await?;\n"
    "    Ok(())\n"
    "}\n\n" + artifacts_functions,
)

gateway_functions = "\n\n".join([fn("built_in_gateway"), fn("built_in_gateway_config")])
gateway_functions = gateway_functions.replace("../../../src-tauri", "../../../../../../src-tauri")
write(
    "crates/application/src/use_cases/research/openai_gateway/gateway.rs",
    "use super::*;\n\n" + gateway_functions,
)

reference_functions = "\n\n".join([
    fn("citation_drafts"),
    fn("source_drafts"),
    fn("verified_reference_domain"),
    fn("optional_u32"),
])
write(
    "crates/application/src/use_cases/research/openai_gateway/references.rs",
    "use super::*;\n\n" + reference_functions,
)

write(
    "crates/application/src/use_cases/research/openai_gateway/attempt_audit.rs",
    '''use super::*;

pub(super) struct PortAttemptSink<'a> {
    pub(super) port: &'a dyn ResearchGatewayAuditPort,
    pub(super) research_run_id: Uuid,
}

#[async_trait]
impl GatewayAttemptSink for PortAttemptSink<'_> {
    async fn record(&self, attempt: &GatewayAttempt) -> Result<(), GatewayError> {
        let error_category = attempt
            .error
            .as_ref()
            .map(|error| error.category.as_str().to_string());
        let error_message = attempt
            .error
            .as_ref()
            .map(|error| error.user_message.clone());
        self.port
            .append_attempt(&OpenAiAttemptDraft {
                research_run_id: self.research_run_id,
                attempt_number: attempt.attempt_number,
                model_id: attempt.model_id.clone(),
                request_fingerprint: attempt.request_fingerprint.clone(),
                request_payload: attempt.request_payload.clone(),
                response_id: attempt.response_id.clone(),
                provider_request_id: attempt.provider_request_id.clone(),
                provider_status: attempt.provider_status,
                status: normalize_attempt_status(&attempt.status),
                token_usage: serde_json::to_value(&attempt.usage).unwrap_or_else(|_| json!({})),
                latency_ms: attempt.latency_ms,
                search_call_count: attempt.search_call_count,
                estimated_cost_usd: attempt.estimated_cost_usd,
                raw_response: attempt.raw_response.clone(),
                error_category,
                error_message,
                retryable: attempt
                    .error
                    .as_ref()
                    .is_some_and(|error| error.recovery.retryable),
                started_at: attempt.started_at,
                finished_at: attempt.finished_at,
            })
            .await
            .map(|_| ())
            .map_err(|error| {
                GatewayError::new(
                    GatewayErrorCategory::Persistence,
                    format!("OpenAI尝试审计写入失败：{error}"),
                    true,
                    "恢复PostgreSQL连接后按原研究任务幂等键重试",
                )
            })
    }
}

pub(super) fn normalize_attempt_status(status: &str) -> String {
    match status {
        "queued" | "in_progress" | "completed" | "failed" | "cancelled" | "incomplete" => {
            status.to_string()
        }
        "succeeded" => "completed".to_string(),
        _ => "failed".to_string(),
    }
}
''',
)

validate = fn("validate_command")
write(
    "crates/application/src/use_cases/research/openai_gateway/validation.rs",
    "use super::*;\n\n" + validate,
)

write(
    "crates/application/src/use_cases/research/openai_gateway/execution.rs",
    '''use super::*;

pub(crate) async fn execute(
    artifacts: &dyn ResearchArtifactPort,
    audit: &dyn ResearchGatewayAuditPort,
    pipeline: &dyn FactPipelineAccess,
    command: OpenAiResearchCommand,
    cancellation: CancellationToken,
) -> ApplicationResult<GatewayExecution> {
    validate_command(&command)?;
    let usage = audit.usage_totals().await?;
    let attempt_number_offset = audit.attempt_number_offset(command.research_run_id).await?;
    let gateway = built_in_gateway()?;
    let schema = built_in_research_schema();
    let prompt = built_in_research_prompt();
    let request = GatewayRequest {
        operation: command.operation,
        trace_id: command.trace_id.to_string(),
        match_key: command.match_key.clone(),
        data_cutoff_at: command.data_cutoff_at,
        schema_name: RESEARCH_SCHEMA_NAME.to_string(),
        schema_version: P4_RESEARCH_OUTPUT_SCHEMA_VERSION.to_string(),
        schema: schema.schema_body,
        static_instructions: prompt.content,
        dynamic_context: command.dynamic_context.clone(),
        requested_fact_keys: command.requested_fact_keys.clone(),
        daily_spend_usd: usage.today_cost_usd,
        monthly_spend_usd: usage.month_cost_usd,
        attempt_number_offset,
    };
    artifacts
        .record_run_event(&ResearchRunEventDraft {
            research_run_id: command.research_run_id,
            idempotency_key: format!(
                "openai:{}:running:{}",
                command.trace_id, attempt_number_offset
            ),
            status: ResearchRunStatus::Running,
            response_id: None,
            model_id: None,
            token_usage: json!({}),
            error_category: None,
            error_message: None,
            payload: json!({
                "gateway_contract": "football.p4-research-gateway.v1",
                "operation": command.operation,
                "schema_version": P4_RESEARCH_OUTPUT_SCHEMA_VERSION,
                "prompt_version": P4_RESEARCH_PROMPT_VERSION,
                "attempt_number_offset": attempt_number_offset
            }),
        })
        .await?;

    let sink = PortAttemptSink {
        port: audit,
        research_run_id: command.research_run_id,
    };
    match gateway
        .execute_with_sink(&request, &cancellation, &sink)
        .await
    {
        Ok(execution) => {
            let retrieved_at = Utc::now();
            let citations = citation_drafts(
                command.research_run_id,
                &execution.response.response_id,
                &execution.response.citations,
                retrieved_at,
            )?;
            let sources = source_drafts(
                command.research_run_id,
                &execution.response.response_id,
                &execution.response.sources,
                retrieved_at,
            )?;
            if let Err(error) = audit
                .append_web_references(command.research_run_id, &sources, &citations)
                .await
            {
                let gateway_error = GatewayError::new(
                    GatewayErrorCategory::Persistence,
                    format!("OpenAI引用持久化失败：{error}"),
                    true,
                    "恢复PostgreSQL连接后按原研究任务幂等键重试；不要重新创建比赛或快照",
                );
                record_gateway_failure(
                    artifacts,
                    &command,
                    &gateway_error,
                    Some(&execution.response.response_id),
                    attempt_number_offset,
                )
                .await?;
                return Err(ApplicationError::ResearchGateway(gateway_error));
            }
            let pipeline_summary = match fact_pipeline::process_p4_research_evidence(
                pipeline,
                ProcessResearchEvidenceCommand {
                    research_run_id: command.research_run_id,
                    response_id: execution.response.response_id.clone(),
                    retrieved_at,
                    output: execution.response.output.clone(),
                    citations: execution.response.citations.clone(),
                    sources: execution.response.sources.clone(),
                },
            )
            .await
            {
                Ok(summary) => summary,
                Err(error) => {
                    artifacts
                        .record_run_event(&ResearchRunEventDraft {
                            research_run_id: command.research_run_id,
                            idempotency_key: format!(
                                "pipeline:{}:{}:failed",
                                command.trace_id, execution.response.response_id
                            ),
                            status: ResearchRunStatus::Partial,
                            response_id: Some(execution.response.response_id.clone()),
                            model_id: Some(execution.response.model_id.clone()),
                            token_usage: serde_json::to_value(&execution.response.usage)?,
                            error_category: Some("fact_pipeline".to_string()),
                            error_message: Some(error.to_string()),
                            payload: json!({
                                "retryable": true,
                                "recommended_action": "修复实体目录、时间字段、来源策略或数据库连接后，按原研究任务幂等键重新执行证据流水线",
                                "strict_schema_validated": true,
                                "rust_validated": true,
                                "references_persisted": true
                            }),
                        })
                        .await?;
                    return Err(ApplicationError::Validation(format!(
                        "联网结果已保存，但实体、时间、冲突或证据路由失败：{error}"
                    )));
                }
            };
            let run_status = if pipeline_summary.has_blockers() {
                ResearchRunStatus::Partial
            } else {
                ResearchRunStatus::Succeeded
            };
            artifacts
                .record_run_event(&ResearchRunEventDraft {
                    research_run_id: command.research_run_id,
                    idempotency_key: format!(
                        "openai:{}:{}:{}",
                        command.trace_id,
                        execution.response.response_id,
                        run_status.as_str()
                    ),
                    status: run_status,
                    response_id: Some(execution.response.response_id.clone()),
                    model_id: Some(execution.response.model_id.clone()),
                    token_usage: serde_json::to_value(&execution.response.usage)?,
                    error_category: None,
                    error_message: None,
                    payload: json!({
                        "provider_request_id": execution.response.provider_request_id.clone(),
                        "citation_count": citations.len(),
                        "source_count": sources.len(),
                        "search_call_count": execution.response.search_call_count,
                        "strict_schema_validated": true,
                        "rust_validated": true,
                        "fact_pipeline": pipeline_summary,
                        "ready_for_stage_f": !pipeline_summary.has_blockers()
                    }),
                })
                .await?;
            Ok(execution)
        }
        Err(error) => {
            record_gateway_failure(
                artifacts,
                &command,
                &error,
                None,
                attempt_number_offset,
            )
            .await?;
            Err(ApplicationError::ResearchGateway(error))
        }
    }
}

async fn record_gateway_failure(
    artifacts: &dyn ResearchArtifactPort,
    command: &OpenAiResearchCommand,
    error: &GatewayError,
    response_id: Option<&str>,
    attempt_number_offset: u32,
) -> ApplicationResult<()> {
    let status = if error.category == GatewayErrorCategory::Cancelled {
        ResearchRunStatus::Cancelled
    } else {
        ResearchRunStatus::Failed
    };
    artifacts
        .record_run_event(&ResearchRunEventDraft {
            research_run_id: command.research_run_id,
            idempotency_key: format!(
                "openai:{}:{}:{}:{}:{}",
                command.trace_id,
                status.as_str(),
                error.category.as_str(),
                response_id.unwrap_or("none"),
                attempt_number_offset
            ),
            status,
            response_id: response_id.map(ToString::to_string),
            model_id: None,
            token_usage: json!({}),
            error_category: Some(error.category.as_str().to_string()),
            error_message: Some(error.user_message.clone()),
            payload: json!({
                "retryable": error.recovery.retryable,
                "recommended_action": error.recovery.action.clone(),
                "provider_status": error.provider_status,
                "provider_code": error.provider_code.clone()
            }),
        })
        .await?;
    Ok(())
}
''',
)

write(
    "crates/application/src/use_cases/research/openai_gateway/mod.rs",
    '''use crate::built_in_artifacts::{
    P4_RESEARCH_PROMPT_ARTIFACT_VERSION, P4_RESEARCH_PROMPT_KEY as RESEARCH_PROMPT_KEY,
    P4_RESEARCH_SCHEMA_ARTIFACT_VERSION, P4_RESEARCH_SCHEMA_KEY as RESEARCH_SCHEMA_KEY,
};
use crate::ports::research::{ResearchArtifactPort, ResearchGatewayAuditPort};
use crate::use_cases::research::fact_pipeline::{self, FactPipelineAccess, ProcessResearchEvidenceCommand};
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

use artifacts::*;
use attempt_audit::*;
use gateway::*;
use references::*;
pub use types::OpenAiResearchCommand;
use validation::*;

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
''',
)
write(
    "crates/application/src/use_cases/research/openai_gateway/tests.rs",
    test_body(),
)

# Add module declaration.
use_cases = ROOT / "crates/application/src/use_cases/research/mod.rs"
text = use_cases.read_text(encoding="utf-8")
if "pub(crate) mod openai_gateway;" not in text:
    text = text.rstrip() + "\npub(crate) mod openai_gateway;\n"
use_cases.write_text(text, encoding="utf-8", newline="\n")

# Extend gateway audit Port with the real offset capability.
ports_path = ROOT / "crates/application/src/ports/research/mod.rs"
text = ports_path.read_text(encoding="utf-8")
anchor = "    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord>;\n    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals>;"
replacement = "    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord>;\n    async fn attempt_number_offset(&self, research_run_id: Uuid) -> PortResult<u32>;\n    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals>;"
if text.count(anchor) != 1:
    raise RuntimeError("ResearchGatewayAuditPort anchor mismatch")
ports_path.write_text(text.replace(anchor, replacement, 1), encoding="utf-8", newline="\n")

# Implement gateway audit adapter on the existing Research adapter.
adapter_path = ROOT / "crates/application/src/composition/adapters/research.rs"
text = adapter_path.read_text(encoding="utf-8")
text = text.replace(
    "FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort,\n        SerializedConflictEventPayload,",
    "FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort, ResearchGatewayAuditPort,\n        SerializedConflictEventPayload,",
    1,
)
text = text.replace(
    "FactPipelineContext, PromptVersionDraft,\n    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,",
    "FactPipelineContext, OpenAiAttemptDraft, OpenAiAttemptRecord, OpenAiUsageTotals, PromptVersionDraft,\n    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,",
    1,
)
text = text.replace(
    "TimeAuditDraft, TimeAuditRecord,\n};",
    "TimeAuditDraft, TimeAuditRecord, WebCitationDraft, WebSourceDraft,\n};",
    1,
)
append_impl = '''

#[async_trait]
impl ResearchGatewayAuditPort for ActiveDatabase {
    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord> {
        self.transition_store()
            .append_openai_attempt(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn attempt_number_offset(&self, research_run_id: Uuid) -> PortResult<u32> {
        self.transition_store()
            .openai_attempt_number_offset(research_run_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals> {
        self.transition_store()
            .openai_usage_totals()
            .await
            .map_err(map_persistence_error)
    }

    async fn append_web_references(
        &self,
        _run_id: Uuid,
        sources: &[WebSourceDraft],
        citations: &[WebCitationDraft],
    ) -> PortResult<()> {
        self.transition_store()
            .append_web_references(citations, sources)
            .await
            .map_err(map_persistence_error)
    }
}
'''
if "impl ResearchGatewayAuditPort for ActiveDatabase" in text:
    raise RuntimeError("ResearchGatewayAuditPort adapter already exists")
adapter_path.write_text(text.rstrip() + append_impl, encoding="utf-8", newline="\n")

# Extend ResearchService without concrete persistence knowledge.
service_path = ROOT / "crates/application/src/services/research/service.rs"
text = service_path.read_text(encoding="utf-8")
text = text.replace(
    "use crate::ports::research::{ResearchArtifactPort, ResearchEvidenceLedgerPort};",
    "use crate::ports::research::{\n    ResearchArtifactPort, ResearchEvidenceLedgerPort, ResearchGatewayAuditPort,\n};",
    1,
)
text = text.replace(
    "    ledger,\n};",
    "    ledger,\n    openai_gateway::{self, OpenAiResearchCommand},\n};",
    1,
)
text = text.replace(
    "use football_domain::{",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\nuse football_domain::{",
    1,
)
insert = '''

    pub(crate) async fn register_openai_research_artifacts(
        &self,
        port: &dyn ResearchArtifactPort,
    ) -> ApplicationResult<()> {
        openai_gateway::register_openai_research_artifacts(port).await
    }

    pub(crate) async fn execute_p4_openai_research(
        &self,
        artifacts: &dyn ResearchArtifactPort,
        audit: &dyn ResearchGatewayAuditPort,
        pipeline: &dyn FactPipelineAccess,
        command: OpenAiResearchCommand,
        cancellation: CancellationToken,
    ) -> ApplicationResult<GatewayExecution> {
        openai_gateway::execute_p4_openai_research(
            artifacts,
            audit,
            pipeline,
            command,
            cancellation,
        )
        .await
    }
'''
pos = text.rfind("\n}")
if pos < 0:
    raise RuntimeError("ResearchService closing brace missing")
service_path.write_text(text[:pos] + insert + text[pos:], encoding="utf-8", newline="\n")

# Public ApplicationService compatibility remains unchanged.
facade_path = ROOT / "crates/application/src/services/research/facade.rs"
text = facade_path.read_text(encoding="utf-8")
text = text.replace(
    "    ApplicationError, ApplicationResult, ApplicationService, ProcessResearchEvidenceCommand,\n};",
    "    ApplicationError, ApplicationResult, ApplicationService, OpenAiResearchCommand,\n    ProcessResearchEvidenceCommand,\n};",
    1,
)
text = text.replace(
    "use football_domain::{",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\nuse football_domain::{",
    1,
)
insert = '''

    pub async fn execute_p4_openai_research(
        &self,
        command: OpenAiResearchCommand,
        cancellation: CancellationToken,
    ) -> ApplicationResult<GatewayExecution> {
        let session = self.research_session().await?;
        self.research
            .execute_p4_openai_research(
                &session,
                &session,
                &session,
                command,
                cancellation,
            )
            .await
    }
'''
pos = text.rfind("\n}")
if pos < 0:
    raise RuntimeError("Research facade closing brace missing")
facade_path.write_text(text[:pos] + insert + text[pos:], encoding="utf-8", newline="\n")

# Database initialization now delegates built-in OpenAI artifacts through ResearchService.
database_path = ROOT / "crates/application/src/services/database/facade.rs"
text = database_path.read_text(encoding="utf-8")
old = '''        let store = prepared.transition_store();
        self.register_openai_research_artifacts(prepared.session(), &store)
            .await
'''
new = '''        self.research
            .register_openai_research_artifacts(prepared.session())
            .await
'''
if text.count(old) != 1:
    raise RuntimeError("database OpenAI initialization anchor mismatch")
database_path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

# Root export moves to the new owner; legacy owner is removed.
lib_path = ROOT / "crates/application/src/lib.rs"
text = lib_path.read_text(encoding="utf-8")
if text.count("mod openai_research;\n") != 1:
    raise RuntimeError("lib openai module anchor mismatch")
text = text.replace("mod openai_research;\n", "", 1)
old = "pub use openai_research::OpenAiResearchCommand;"
new = "pub use use_cases::research::openai_gateway::OpenAiResearchCommand;"
if text.count(old) != 1:
    raise RuntimeError("OpenAiResearchCommand export anchor mismatch")
lib_path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

# Research verifier moves ownership checks from the legacy file to the new modules.
verifier_path = ROOT / "scripts/verify-research-service.mjs"
text = verifier_path.read_text(encoding="utf-8")
text = text.replace('  "process_p4_research_evidence",\n];', '  "process_p4_research_evidence",\n  "execute_p4_openai_research",\n];', 1)
required_anchor = '  "crates/application/src/use_cases/research/fact_pipeline/types.rs",\n'
required_add = required_anchor + ''.join(
    f'  "crates/application/src/use_cases/research/openai_gateway/{name}",\n'
    for name in ["mod.rs", "artifacts.rs", "attempt_audit.rs", "execution.rs", "gateway.rs", "references.rs", "types.rs", "validation.rs"]
)
if text.count(required_anchor) != 1:
    raise RuntimeError("Research verifier required-files anchor mismatch")
text = text.replace(required_anchor, required_add, 1)
text = text.replace(
    'check(!existsSync(join(root, "crates/application/src/fact_pipeline.rs")), "旧 fact_pipeline.rs 仍残留 Fact Pipeline owner 或空转发层");',
    'check(!existsSync(join(root, "crates/application/src/fact_pipeline.rs")), "旧 fact_pipeline.rs 仍残留 Fact Pipeline owner 或空转发层");\ncheck(!existsSync(join(root, "crates/application/src/openai_research.rs")), "旧 openai_research.rs 仍残留 OpenAI Research owner 或空转发层");',
    1,
)
text = text.replace(
    'const openai = read("crates/application/src/openai_research.rs");',
    'const openaiExecution = read("crates/application/src/use_cases/research/openai_gateway/execution.rs");\nconst openaiArtifacts = read("crates/application/src/use_cases/research/openai_gateway/artifacts.rs");\nconst openaiAudit = read("crates/application/src/use_cases/research/openai_gateway/attempt_audit.rs");',
    1,
)
old_check = 'check(openai.replaceAll(" ", "").replaceAll(String.fromCharCode(10), "").replaceAll(String.fromCharCode(13), "").replaceAll(String.fromCharCode(9), "").includes("self.research.register_fact_pipeline_artifacts(session).await?"), "OpenAI artifact 初始化未通过 ResearchService 注册 Fact Pipeline 来源策略");\ncheck(openai.includes("fn execute_p4_openai_research"), "后续 OpenAI Research 执行职责被提前迁移或删除");'
new_check = '''check(openaiArtifacts.includes("fact_pipeline::register_fact_pipeline_artifacts(port).await?"), "OpenAI artifact 初始化未继续注册 Fact Pipeline 来源策略");
check(service.includes("fn register_openai_research_artifacts"), "ResearchService 缺少 OpenAI artifact 初始化职责");
check(databaseFacade.replaceAll(/\\s/g, "").includes("self.research.register_openai_research_artifacts(prepared.session()).await"), "数据库初始化未通过 ResearchService 注册 OpenAI Research artifacts");
check(ports.includes("trait ResearchGatewayAuditPort"), "Research Ports 缺少 ResearchGatewayAuditPort");
for (const capability of ["append_attempt", "attempt_number_offset", "usage_totals", "append_web_references"]) {
  check(ports.includes(`fn ${capability}`), `ResearchGatewayAuditPort 缺少能力：${capability}`);
}
check(adapter.includes("impl ResearchGatewayAuditPort for ActiveDatabase"), "Research adapter 缺少 Gateway Audit 实现");
for (const token of [".append_openai_attempt(draft)", ".openai_attempt_number_offset(research_run_id)", ".openai_usage_totals()", ".append_web_references(citations, sources)"]) {
  check(adapter.includes(token), `Research Gateway adapter 未复用既有持久化能力：${token}`);
}
check(openaiExecution.includes("execute_with_sink"), "OpenAI Research 执行未保留 GatewayAttemptSink 审计链");
check(openaiExecution.includes("fact_pipeline::process_p4_research_evidence"), "OpenAI Research 执行未衔接 Fact Pipeline");
check(openaiAudit.includes("impl GatewayAttemptSink for PortAttemptSink"), "OpenAI attempt audit 未通过 Port-backed sink");'''
if text.count(old_check) != 1:
    raise RuntimeError("Research verifier legacy OpenAI checks anchor mismatch")
text = text.replace(old_check, new_check, 1)
text = text.replace(
    'for (const path of ["crates/application/src/openai_research.rs", "crates/application/src/p4_orchestration.rs", "crates/application/src/p4_workbench.rs"]) {',
    'for (const path of ["crates/application/src/p4_orchestration.rs", "crates/application/src/p4_workbench.rs"]) {',
    1,
)
text = text.replace(
    'check(lib.includes("use_cases::research::fact_pipeline::ProcessResearchEvidenceCommand"), "公共 ProcessResearchEvidenceCommand 未从新 owner 重导出");',
    'check(lib.includes("use_cases::research::fact_pipeline::ProcessResearchEvidenceCommand"), "公共 ProcessResearchEvidenceCommand 未从新 owner 重导出");\ncheck(lib.includes("use_cases::research::openai_gateway::OpenAiResearchCommand"), "公共 OpenAiResearchCommand 未从新 owner 重导出");\ncheck(!lib.includes("mod openai_research;"), "Application 根模块仍登记旧 openai_research owner");',
    1,
)
text = re.sub(
    r'console\.log\(`Research Service AT2 验证通过：.*?`\);',
    'console.log(`Research Service AT3 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件，9 个公开 Research API 已进入 ResearchService/Ports；Fact Pipeline 保持 ${pipelineFiles.length} 个模块，OpenAI Gateway execution 已拆入 ResearchService，旧 p4_persistence.rs / fact_pipeline.rs / openai_research.rs 均已删除。`);',
    text,
    count=1,
    flags=re.S,
)
verifier_path.write_text(text, encoding="utf-8", newline="\n")

# Delete the old owner only after all replacement paths are generated.
LEGACY.unlink()

# README records actual AT3 implementation, not completion.
def insert_after(path: Path, needle: str, line: str) -> None:
    text = path.read_text(encoding="utf-8")
    if line in text:
        return
    lines = text.splitlines()
    for index, current in enumerate(lines):
        if needle in current:
            lines.insert(index + 1, line)
            path.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
            return
    raise RuntimeError(f"README anchor not found: {needle}")

at3_record = "- R3-07 Atomic Task 3 已迁移 OpenAI Research Gateway execution：旧 `crates/application/src/openai_research.rs` 删除；网关 artifacts、配置、attempt audit、引用转换、执行状态机与输入验证按职责拆入 `use_cases/research/openai_gateway/`，公共 `execute_p4_openai_research` / `OpenAiResearchCommand` 契约保持不变。`ResearchGatewayAuditPort` 补齐既有 attempt-number offset 能力并由 `ActiveDatabase` 适配现有 PostgreSQL 方法；数据库 Schema、迁移、生产依赖、research-gateway transport 与 P4 worker / 人工冲突写入行为未改变。AT3 仍处于 `IN_PROGRESS`，须通过 staging 全量硬门禁及正式 Public Platform CI 后才能关闭。"
insert_after(ROOT / "README.md", "Atomic Task 2 已正式关闭为 `DONE`", at3_record)

r3_record = "- Atomic Task 3 已迁移 OpenAI Research Gateway execution：删除旧 `openai_research.rs`，按 artifacts / gateway / attempt audit / references / execution / validation 职责拆入 Research use cases；公共命令与错误语义保持不变。现有 `ResearchGatewayAuditPort` 仅补齐 attempt-number offset，并由 `ActiveDatabase` 复用既有 PostgreSQL gateway records。P4 orchestration worker 与 manual conflict mutation 保持后续 Atomic Task 边界。AT3 当前 `IN_PROGRESS`，正式 Public Platform CI 通过前不得标记 `DONE`。"
insert_after(ROOT / "docs/modular-rewrite/R03-application-services/README.md", "Atomic Task 2 已正式关闭为 `DONE`", r3_record)

print("R3-07 Atomic Task 3 OpenAI Research Gateway migration generated")
