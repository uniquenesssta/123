from pathlib import Path
import os
import re
import textwrap

ROOT = Path(__file__).resolve().parents[1]
RUN_ID = os.environ.get("GITHUB_RUN_ID", "unknown")
LEGACY = ROOT / "crates/application/src/fact_pipeline.rs"


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content.rstrip() + "\n", encoding="utf-8", newline="\n")


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise RuntimeError(f"expected exactly one match in {path}: {old!r}; count={text.count(old)}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


def extract_braced(text: str, marker: str, include_attributes: bool = True) -> str:
    marker_index = text.find(marker)
    if marker_index < 0:
        raise RuntimeError(f"marker not found: {marker}")
    line_start = text.rfind("\n", 0, marker_index) + 1
    start = line_start
    if include_attributes:
        while start > 0:
            prev_end = start - 1
            prev_start = text.rfind("\n", 0, prev_end) + 1
            prev_line = text[prev_start:prev_end].strip()
            if prev_line.startswith("#["):
                start = prev_start
            else:
                break
    open_brace = text.find("{", marker_index)
    if open_brace < 0:
        raise RuntimeError(f"opening brace not found: {marker}")
    depth = 0
    in_string = False
    escape = False
    for index in range(open_brace, len(text)):
        char = text[index]
        if in_string:
            if escape:
                escape = False
            elif char == "\\":
                escape = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
            continue
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[start:index + 1]
    raise RuntimeError(f"unclosed braced item: {marker}")


def extract_function(text: str, name: str) -> str:
    match = re.search(rf"(?m)^(?:async\s+)?fn\s+{re.escape(name)}\s*\(", text)
    if not match:
        raise RuntimeError(f"top-level function not found: {name}")
    return extract_braced(text, match.group(0).strip(), include_attributes=True)


def make_pub_super_function(block: str, name: str) -> str:
    pattern = re.compile(rf"(?m)^(async\s+)?fn\s+{re.escape(name)}\s*\(")
    replaced, count = pattern.subn(lambda m: f"pub(super) {m.group(1) or ''}fn {name}(", block, count=1)
    if count != 1:
        raise RuntimeError(f"failed to set function visibility: {name}")
    return replaced


def extract_struct(text: str, name: str, public: bool = False) -> str:
    block = extract_braced(text, f"struct {name}", include_attributes=True)
    if public:
        return block
    block, count = re.subn(rf"(?m)^struct\s+{re.escape(name)}\s*\{{", f"pub(super) struct {name} {{", block, count=1)
    if count != 1:
        raise RuntimeError(f"failed to set struct visibility: {name}")
    lines = []
    for line in block.splitlines():
        if re.match(r"^    [A-Za-z_][A-Za-z0-9_]*\s*:", line):
            line = "    pub(super) " + line[4:]
        lines.append(line)
    return "\n".join(lines)


source = LEGACY.read_text(encoding="utf-8")

function_groups = {
    "entity_resolution.rs": [
        "decide_entity_resolution",
        "normalize_entity_name",
        "compact_entity_name",
    ],
    "time_audit.rs": ["audit_fact_time"],
    "source_policy.rs": [
        "build_source_index",
        "classify_source",
        "domain_matches",
        "normalize_domain",
        "normalize_source_url",
        "normalize_url",
        "built_in_source_policy",
    ],
    "evidence.rs": [
        "process_fact_group",
        "append_fact_claims",
        "independent_domains_by_value",
        "max_rank_by_value",
        "determine_claim_state",
    ],
    "conflict.rs": [
        "process_conflict_group",
        "rank_values",
        "compare_ranked_values",
        "conflict_winner",
    ],
    "routing.rs": [
        "route_non_conflicting_group",
        "resolved_route_status",
        "append_route",
        "process_missing_field",
        "built_in_route_registry",
        "validate_route_registry",
    ],
    "validation.rs": [
        "verification_priority",
        "parse_verification_state",
        "canonical_json",
        "sha256_text",
        "validate_pipeline_command",
        "validate_pipeline_context",
    ],
}
assigned = {name for names in function_groups.values() for name in names}
tests_start = source.index("#[cfg(test)]\nmod tests {")
impl_start = source.index("impl ApplicationService {")
impl_end_search = source.index("\nasync fn process_fact_group", impl_start)
legacy_top_level = source[impl_end_search:tests_start]
discovered = set(re.findall(r"(?m)^(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", legacy_top_level))
if discovered != assigned:
    raise RuntimeError(
        "Fact Pipeline top-level function classification mismatch; "
        f"missing={sorted(discovered - assigned)}, extra={sorted(assigned - discovered)}"
    )

command_struct = extract_struct(source, "ProcessResearchEvidenceCommand", public=True)
internal_structs = "\n\n".join(
    extract_struct(source, name)
    for name in ["SourceReference", "PreparedFact", "PersistedFact", "RankedValue", "ResolutionDecision"]
)
write("crates/application/src/use_cases/research/fact_pipeline/types.rs", internal_structs)

for filename, names in function_groups.items():
    blocks = []
    for name in names:
        block = make_pub_super_function(extract_function(source, name), name)
        if filename in {"source_policy.rs", "routing.rs"}:
            block = block.replace("../../../src-tauri", "../../../../../../src-tauri")
        blocks.append(block)
    write(
        f"crates/application/src/use_cases/research/fact_pipeline/{filename}",
        "use super::*;\n\n" + "\n\n".join(blocks),
    )

process_method = extract_braced(source, "pub async fn process_p4_research_evidence", include_attributes=False)
open_brace = process_method.find("{")
process_body = process_method[open_brace + 1:-1]
process_body = process_body.replace("        let store = self.active_store().await?;\n", "", 1)
process_body = re.sub(r"\bstore\b", "port", process_body)
process_body = textwrap.indent(textwrap.dedent(process_body).strip("\n"), "    ")

test_module = extract_braced(source, "mod tests {", include_attributes=True)
test_open = test_module.find("{")
test_body = textwrap.dedent(test_module[test_open + 1:-1]).strip("\n")
write("crates/application/src/use_cases/research/fact_pipeline/tests.rs", test_body)

header = f'''use crate::ports::research::{{
    FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort,
    SerializedConflictEventPayload,
}};
use crate::{{ApplicationError, ApplicationResult}};
use chrono::{{DateTime, Utc}};
use football_domain::{{
    ConflictEvaluationDraft, ConflictEvaluationStatus, EntityCandidate, EntityResolutionDraft,
    EntityResolutionRecord, EntityResolutionStatus, EvidenceClaimDraft, EvidenceClaimRecord,
    EvidenceConflictDraft, EvidenceRouteDraft, EvidenceRouteRegistry, EvidenceRouteRule,
    EvidenceRouteStatus, EvidenceVerificationState, FactPipelineContext, FactPipelineSummary,
    SourcePolicyDefinition, SourcePolicyVersionDraft, TimeAuditDraft, TimeAuditRecord,
    TimeAuditStatus, P4_EVIDENCE_ROUTE_VERSION, P4_SOURCE_POLICY_VERSION,
}};
use football_research_gateway::{{
    MissingField, ResearchFact, ResearchOutput, WebCitation, WebSource,
}};
use serde::{{Deserialize, Serialize}};
use serde_json::{{json, Value}};
use sha2::{{Digest, Sha256}};
use std::cmp::Ordering;
use std::collections::{{BTreeMap, BTreeSet}};
use url::Url;
use uuid::Uuid;

mod conflict;
mod entity_resolution;
mod evidence;
mod routing;
mod source_policy;
mod time_audit;
mod types;
mod validation;

use conflict::*;
use entity_resolution::*;
use evidence::*;
use routing::*;
use source_policy::*;
use time_audit::*;
use types::*;
use validation::*;

const SOURCE_POLICY_KEY: &str = "p4-default-source-policy";
const SOURCE_POLICY_SEMVER: &str = "1.0.0";

{command_struct}

pub(crate) trait FactPipelineAccess: FactPipelinePort + ResearchEvidenceLedgerPort {{}}
impl<T> FactPipelineAccess for T where T: FactPipelinePort + ResearchEvidenceLedgerPort + ?Sized {{}}

pub(crate) async fn register_fact_pipeline_artifacts(
    port: &dyn ResearchArtifactPort,
) -> ApplicationResult<()> {{
    port.register_source_policy(&built_in_source_policy()).await?;
    validate_route_registry(&built_in_route_registry())?;
    Ok(())
}}

pub(crate) async fn process_p4_research_evidence(
    port: &dyn FactPipelineAccess,
    command: ProcessResearchEvidenceCommand,
) -> ApplicationResult<FactPipelineSummary> {{
{process_body}
}}

#[cfg(test)]
mod tests;
'''
write("crates/application/src/use_cases/research/fact_pipeline/mod.rs", header)

moved_dir = ROOT / "crates/application/src/use_cases/research/fact_pipeline"
for path in moved_dir.glob("*.rs"):
    if path.name == "types.rs":
        continue
    text = path.read_text(encoding="utf-8")
    text = re.sub(r"\bPersistenceStore\b", "dyn FactPipelineAccess", text)
    text = re.sub(r"\bstore\b", "port", text)
    text = text.replace("port.fact_pipeline_context(", "port.context(")
    text = text.replace("port.register_source_policy_version(", "port.register_source_policy(")
    path.write_text(text, encoding="utf-8", newline="\n")

conflict_path = moved_dir / "conflict.rs"
conflict_text = conflict_path.read_text(encoding="utf-8")
pattern = re.compile(
    r'''port\s*\.append_conflict_event\(\s*conflict\.id,\s*"resolved",\s*"deterministic-conflict-resolver",\s*&json!\((\{.*?\})\),\s*"deterministic-resolution-v1",\s*\)\s*\.await\?;''',
    re.S,
)
match = pattern.search(conflict_text)
if not match:
    raise RuntimeError("conflict event payload call not found for typed Port conversion")
payload = match.group(1)
replacement = f'''let conflict_event_payload = SerializedConflictEventPayload(
            serde_json::to_string(&json!({payload}))?,
        );
        port.append_conflict_event(
            conflict.id,
            "resolved",
            "deterministic-conflict-resolver",
            &conflict_event_payload,
            "deterministic-resolution-v1",
        )
        .await?;'''
conflict_text = conflict_text[:match.start()] + replacement + conflict_text[match.end():]
conflict_path.write_text(conflict_text, encoding="utf-8", newline="\n")

write(
    "crates/application/src/ports/research/mod.rs",
    '''use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, ConflictEvaluationDraft,
    ConflictEvaluationRecord, EntityCandidate, EntityResolutionDraft, EntityResolutionRecord,
    EvidenceClaimDraft, EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord,
    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, OpenAiAttemptDraft,
    OpenAiAttemptRecord, OpenAiUsageTotals, PromptVersionDraft, PromptVersionRecord,
    ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft,
    SchemaVersionRecord, SourcePolicyVersionDraft, SourcePolicyVersionRecord, TimeAuditDraft,
    TimeAuditRecord, WebCitationDraft, WebSourceDraft,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedConflictEventPayload(pub String);

#[async_trait]
pub trait ResearchArtifactPort: Send + Sync {
    async fn read_schema(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PortResult<SchemaVersionRecord>;
    async fn register_schema(&self, draft: &SchemaVersionDraft) -> PortResult<SchemaVersionRecord>;
    async fn register_prompt(&self, draft: &PromptVersionDraft) -> PortResult<PromptVersionRecord>;
    async fn register_source_policy(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord>;
    async fn register_competition_profile(
        &self,
        draft: &CompetitionProfileVersionDraft,
    ) -> PortResult<CompetitionProfileVersionRecord>;
    async fn create_run(&self, draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord>;
    async fn read_run(&self, run_id: Uuid) -> PortResult<ResearchRunRecord>;
    async fn record_run_event(
        &self,
        draft: &ResearchRunEventDraft,
    ) -> PortResult<ResearchRunRecord>;
}

#[async_trait]
pub trait ResearchEvidenceLedgerPort: Send + Sync {
    async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PortResult<EvidenceClaimRecord>;
    async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PortResult<EvidenceConflictRecord>;
}

#[async_trait]
pub trait FactPipelinePort: Send + Sync {
    async fn context(&self, research_run_id: Uuid) -> PortResult<FactPipelineContext>;
    async fn find_entity_candidates(
        &self,
        context: &FactPipelineContext,
        entity_type: &str,
        normalized_name: &str,
        compact_name: &str,
        external_id: Option<&str>,
    ) -> PortResult<Vec<EntityCandidate>>;
    async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PortResult<EntityResolutionRecord>;
    async fn append_time_audit(&self, draft: &TimeAuditDraft) -> PortResult<TimeAuditRecord>;
    async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PortResult<ConflictEvaluationRecord>;
    async fn append_conflict_event(
        &self,
        conflict_id: Uuid,
        event_type: &str,
        actor: &str,
        payload: &SerializedConflictEventPayload,
        idempotency_key: &str,
    ) -> PortResult<()>;
    async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PortResult<EvidenceRouteRecord>;
}

#[async_trait]
pub trait ResearchGatewayAuditPort: Send + Sync {
    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord>;
    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals>;
    async fn append_web_references(
        &self,
        run_id: Uuid,
        sources: &[WebSourceDraft],
        citations: &[WebCitationDraft],
    ) -> PortResult<()>;
}
''',
)

write(
    "crates/application/src/composition/adapters/research.rs",
    '''use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{
    research::{
        FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort,
        SerializedConflictEventPayload,
    },
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, ConflictEvaluationDraft,
    ConflictEvaluationRecord, EntityCandidate, EntityResolutionDraft, EntityResolutionRecord,
    EvidenceClaimDraft, EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord,
    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, PromptVersionDraft,
    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
    SchemaVersionDraft, SchemaVersionRecord, SourcePolicyVersionDraft, SourcePolicyVersionRecord,
    TimeAuditDraft, TimeAuditRecord,
};
use uuid::Uuid;

#[async_trait]
impl ResearchArtifactPort for ActiveDatabase {
    async fn read_schema(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PortResult<SchemaVersionRecord> {
        self.transition_store()
            .read_schema_version_by_key(schema_key, version)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_schema(&self, draft: &SchemaVersionDraft) -> PortResult<SchemaVersionRecord> {
        self.transition_store()
            .register_schema_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_prompt(&self, draft: &PromptVersionDraft) -> PortResult<PromptVersionRecord> {
        self.transition_store()
            .register_prompt_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_source_policy(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord> {
        self.transition_store()
            .register_source_policy_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_competition_profile(
        &self,
        draft: &CompetitionProfileVersionDraft,
    ) -> PortResult<CompetitionProfileVersionRecord> {
        self.transition_store()
            .register_competition_profile_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_run(&self, draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .create_research_run(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_run(&self, run_id: Uuid) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .read_research_run(run_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn record_run_event(
        &self,
        draft: &ResearchRunEventDraft,
    ) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .record_research_run_event(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ResearchEvidenceLedgerPort for ActiveDatabase {
    async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PortResult<EvidenceClaimRecord> {
        self.transition_store()
            .append_evidence_claim(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PortResult<EvidenceConflictRecord> {
        self.transition_store()
            .create_evidence_conflict(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl FactPipelinePort for ActiveDatabase {
    async fn context(&self, research_run_id: Uuid) -> PortResult<FactPipelineContext> {
        self.transition_store()
            .fact_pipeline_context(research_run_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn find_entity_candidates(
        &self,
        context: &FactPipelineContext,
        entity_type: &str,
        normalized_name: &str,
        compact_name: &str,
        external_id: Option<&str>,
    ) -> PortResult<Vec<EntityCandidate>> {
        self.transition_store()
            .find_entity_candidates(
                context,
                entity_type,
                normalized_name,
                compact_name,
                external_id,
            )
            .await
            .map_err(map_persistence_error)
    }

    async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PortResult<EntityResolutionRecord> {
        self.transition_store()
            .append_entity_resolution(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_time_audit(&self, draft: &TimeAuditDraft) -> PortResult<TimeAuditRecord> {
        self.transition_store()
            .append_time_audit(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PortResult<ConflictEvaluationRecord> {
        self.transition_store()
            .append_conflict_evaluation(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_conflict_event(
        &self,
        conflict_id: Uuid,
        event_type: &str,
        actor: &str,
        payload: &SerializedConflictEventPayload,
        idempotency_key: &str,
    ) -> PortResult<()> {
        let payload = serde_json::from_str(&payload.0).map_err(|error| {
            PortError::new(
                PortErrorKind::Serialization,
                format!("冲突事件载荷反序列化失败：{error}"),
            )
        })?;
        self.transition_store()
            .append_conflict_event(conflict_id, event_type, actor, &payload, idempotency_key)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PortResult<EvidenceRouteRecord> {
        self.transition_store()
            .append_evidence_route(draft)
            .await
            .map_err(map_persistence_error)
    }
}
''',
)

replace_once(
    "crates/application/src/use_cases/research/mod.rs",
    "pub(crate) mod artifact_catalog;\npub(crate) mod ledger;",
    "pub(crate) mod artifact_catalog;\npub(crate) mod fact_pipeline;\npub(crate) mod ledger;",
)

service_path = ROOT / "crates/application/src/services/research/service.rs"
service = service_path.read_text(encoding="utf-8")
service = service.replace(
    "use crate::use_cases::research::{artifact_catalog, ledger};",
    "use crate::use_cases::research::{\n    artifact_catalog,\n    fact_pipeline::{self, FactPipelineAccess, ProcessResearchEvidenceCommand},\n    ledger,\n};",
    1,
)
service = service.replace(
    "    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PromptVersionDraft,",
    "    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, FactPipelineSummary,\n    PromptVersionDraft,",
    1,
)
service_insert = '''

    pub(crate) async fn register_fact_pipeline_artifacts(
        &self,
        port: &dyn ResearchArtifactPort,
    ) -> ApplicationResult<()> {
        fact_pipeline::register_fact_pipeline_artifacts(port).await
    }

    pub(crate) async fn process_p4_research_evidence(
        &self,
        port: &dyn FactPipelineAccess,
        command: ProcessResearchEvidenceCommand,
    ) -> ApplicationResult<FactPipelineSummary> {
        fact_pipeline::process_p4_research_evidence(port, command).await
    }
'''
last = service.rfind("\n}")
if last < 0:
    raise RuntimeError("ResearchService closing brace not found")
service = service[:last] + service_insert + service[last:]
service_path.write_text(service, encoding="utf-8", newline="\n")

facade_path = ROOT / "crates/application/src/services/research/facade.rs"
facade = facade_path.read_text(encoding="utf-8")
facade = facade.replace(
    "use crate::{ApplicationError, ApplicationResult, ApplicationService};",
    "use crate::{\n    ApplicationError, ApplicationResult, ApplicationService, ProcessResearchEvidenceCommand,\n};",
    1,
)
facade = facade.replace(
    "    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PromptVersionDraft,",
    "    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, FactPipelineSummary,\n    PromptVersionDraft,",
    1,
)
facade_insert = '''

    pub async fn process_p4_research_evidence(
        &self,
        command: ProcessResearchEvidenceCommand,
    ) -> ApplicationResult<FactPipelineSummary> {
        let session = self.research_session().await?;
        self.research
            .process_p4_research_evidence(&session, command)
            .await
    }
'''
last = facade.rfind("\n}")
if last < 0:
    raise RuntimeError("Research facade closing brace not found")
facade = facade[:last] + facade_insert + facade[last:]
facade_path.write_text(facade, encoding="utf-8", newline="\n")

replace_once(
    "crates/application/src/lib.rs",
    "mod fact_pipeline;\n",
    "",
)
replace_once(
    "crates/application/src/lib.rs",
    "pub use fact_pipeline::ProcessResearchEvidenceCommand;",
    "pub use use_cases::research::fact_pipeline::ProcessResearchEvidenceCommand;",
)

replace_once(
    "crates/application/src/openai_research.rs",
    "use super::{ApplicationError, ApplicationResult, ApplicationService, PersistenceStore};",
    "use super::{ApplicationError, ApplicationResult, ApplicationService, PersistenceStore};\nuse crate::composition::ActiveDatabase;",
)
replace_once(
    "crates/application/src/openai_research.rs",
    "    pub(super) async fn register_openai_research_artifacts(\n        &self,\n        store: &PersistenceStore,\n    ) -> ApplicationResult<()> {",
    "    pub(super) async fn register_openai_research_artifacts(\n        &self,\n        session: &ActiveDatabase,\n        store: &PersistenceStore,\n    ) -> ApplicationResult<()> {",
)
replace_once(
    "crates/application/src/openai_research.rs",
    "        self.register_fact_pipeline_artifacts(store).await?;",
    "        self.research.register_fact_pipeline_artifacts(session).await?;",
)
replace_once(
    "crates/application/src/services/database/facade.rs",
    "        self.register_openai_research_artifacts(&store).await",
    "        self.register_openai_research_artifacts(prepared.session(), &store)\n            .await",
)

if not LEGACY.exists():
    raise RuntimeError("legacy fact_pipeline.rs missing before AT2 migration")
LEGACY.unlink()

write(
    "scripts/verify-research-service.mjs",
    '''import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replaceAll("\\r\\n", "\\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
function rustFiles(path) {
  const absolute = join(root, path);
  const result = [];
  for (const entry of readdirSync(absolute, { withFileTypes: true })) {
    const child = join(absolute, entry.name);
    if (entry.isDirectory()) result.push(...rustFiles(relative(root, child)));
    else if (entry.name.endsWith(".rs")) result.push(relative(root, child).replaceAll("\\\\", "/"));
  }
  return result;
}

const publicMethods = [
  "register_p4_schema_version",
  "register_p4_prompt_version",
  "register_p4_competition_profile_version",
  "create_p4_research_run",
  "record_p4_research_run_event",
  "append_p4_evidence_claim",
  "create_p4_evidence_conflict",
  "process_p4_research_evidence",
];
const required = [
  "crates/application/src/services/research/mod.rs",
  "crates/application/src/services/research/service.rs",
  "crates/application/src/services/research/facade.rs",
  "crates/application/src/use_cases/research/mod.rs",
  "crates/application/src/use_cases/research/artifact_catalog.rs",
  "crates/application/src/use_cases/research/ledger.rs",
  "crates/application/src/use_cases/research/fact_pipeline/mod.rs",
  "crates/application/src/use_cases/research/fact_pipeline/entity_resolution.rs",
  "crates/application/src/use_cases/research/fact_pipeline/time_audit.rs",
  "crates/application/src/use_cases/research/fact_pipeline/source_policy.rs",
  "crates/application/src/use_cases/research/fact_pipeline/evidence.rs",
  "crates/application/src/use_cases/research/fact_pipeline/conflict.rs",
  "crates/application/src/use_cases/research/fact_pipeline/routing.rs",
  "crates/application/src/use_cases/research/fact_pipeline/validation.rs",
  "crates/application/src/use_cases/research/fact_pipeline/types.rs",
  "crates/application/src/composition/adapters/research.rs",
  "crates/application/src/ports/research/mod.rs",
];
for (const path of required) check(existsSync(join(root, path)), `缺少 R3-07 文件：${path}`);
check(!existsSync(join(root, "crates/application/src/p4_persistence.rs")), "旧 p4_persistence.rs 仍残留 Research owner");
check(!existsSync(join(root, "crates/application/src/fact_pipeline.rs")), "旧 fact_pipeline.rs 仍残留 Fact Pipeline owner 或空转发层");

const facade = read("crates/application/src/services/research/facade.rs");
const service = read("crates/application/src/services/research/service.rs");
const ports = read("crates/application/src/ports/research/mod.rs");
const adapter = read("crates/application/src/composition/adapters/research.rs");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
const openai = read("crates/application/src/openai_research.rs");
const pipeline = read("crates/application/src/use_cases/research/fact_pipeline/mod.rs");
const lib = read("crates/application/src/lib.rs");
const packageJson = JSON.parse(read("package.json"));
const frontend = read("scripts/verify-frontend.mjs");

for (const method of publicMethods) {
  check(facade.includes(`fn ${method}`), `Research facade 缺少公共兼容方法：${method}`);
  check(service.includes(`fn ${method}`), `ResearchService 缺少职责：${method}`);
}
check(ports.includes("trait ResearchArtifactPort"), "Research Ports 缺少 ResearchArtifactPort");
check(ports.includes("register_competition_profile"), "ResearchArtifactPort 缺少赛事配置版本写入能力");
check(ports.includes("PortResult<ResearchRunRecord>"), "Research run event 未保留公开返回记录契约");
check(ports.includes("trait ResearchEvidenceLedgerPort"), "Research Ports 缺少 ResearchEvidenceLedgerPort");
check(ports.includes("trait FactPipelinePort"), "Research Ports 缺少 FactPipelinePort");
check(ports.includes("struct SerializedConflictEventPayload"), "FactPipelinePort 缺少类型化冲突事件载荷边界");
check(!ports.includes("serde_json::Value"), "Research Ports 泄漏裸 serde_json::Value");
for (const capability of ["find_entity_candidates", "append_entity_resolution", "append_time_audit", "append_conflict_evaluation", "append_conflict_event", "append_evidence_route"]) {
  check(ports.includes(`fn ${capability}`), `FactPipelinePort 缺少能力：${capability}`);
}
check(adapter.includes("impl ResearchEvidenceLedgerPort for ActiveDatabase"), "Research adapter 缺少 Evidence Ledger 实现");
check(adapter.includes("impl FactPipelinePort for ActiveDatabase"), "Research adapter 缺少 Fact Pipeline 实现");
for (const token of [".fact_pipeline_context(research_run_id)", ".find_entity_candidates(", ".append_entity_resolution(draft)", ".append_time_audit(draft)", ".append_conflict_evaluation(draft)", ".append_conflict_event(", ".append_evidence_route(draft)"]) {
  check(adapter.includes(token), `Fact Pipeline adapter 未复用既有持久化能力：${token}`);
}
check(databaseFacade.includes(".register_persistence_artifacts(prepared.session())"), "数据库初始化未通过 ResearchService 注册内置 Research schema");
check(openai.includes("self.research.register_fact_pipeline_artifacts(session).await?"), "OpenAI artifact 初始化未通过 ResearchService 注册 Fact Pipeline 来源策略");
check(openai.includes("fn execute_p4_openai_research"), "后续 OpenAI Research 执行职责被提前迁移或删除");
check(!lib.includes("mod p4_persistence;"), "Application 根模块仍登记旧 p4_persistence owner");
check(!lib.includes("mod fact_pipeline;"), "Application 根模块仍登记旧 fact_pipeline owner");
check(lib.includes("use_cases::research::fact_pipeline::ProcessResearchEvidenceCommand"), "公共 ProcessResearchEvidenceCommand 未从新 owner 重导出");
check(pipeline.includes("trait FactPipelineAccess"), "Fact Pipeline 缺少 Ports 组合访问边界");
for (const path of ["crates/application/src/openai_research.rs", "crates/application/src/p4_orchestration.rs", "crates/application/src/p4_workbench.rs"]) {
  check(existsSync(join(root, path)), `后续 R3-07 职责被提前删除：${path}`);
}

const pipelineFiles = rustFiles("crates/application/src/use_cases/research/fact_pipeline");
check(pipelineFiles.length >= 9, `Fact Pipeline 拆分不足，当前仅 ${pipelineFiles.length} 个职责文件`);
const researchFiles = [
  ...rustFiles("crates/application/src/services/research"),
  ...rustFiles("crates/application/src/use_cases/research"),
];
for (const path of researchFiles) {
  const source = read(path);
  for (const token of ["football_persistence_postgres", "PostgresStore", "sqlx::", "PgPool", "PersistenceStore"]) {
    check(!source.includes(token), `${path} 泄漏具体持久化实现：${token}`);
  }
}
check(packageJson.scripts?.["verify:research-service"] === "node scripts/verify-research-service.mjs", "package.json 未登记 R3-07 专项门禁");
check(packageJson.scripts?.["verify:architecture"]?.includes("verify-research-service.mjs"), "verify:architecture 未接入 R3-07 门禁");
check(frontend.includes('"verify-research-service.mjs"'), "verify:frontend 未接入 R3-07 门禁");

if (failures.length) throw new Error(`Research Service 验证失败\\n${failures.map((item) => `- ${item}`).join("\\n")}`);
console.log(`Research Service AT2 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件，8 个公开 Research API 已进入 ResearchService/Ports；Fact Pipeline 已按职责拆分为 ${pipelineFiles.length} 个模块，旧 p4_persistence.rs / fact_pipeline.rs 均已删除。`);
''',
)

root_path = ROOT / "README.md"
root = root_path.read_text(encoding="utf-8")
anchor = '- Atomic Task 1 已正式关闭为 `DONE`。clean 正式 HEAD `ad53bd6c8cdbd93c9e58a38642ae4d22c7d32df7` 的 Public Platform CI run `31296372108` / Windows Automated job `93201993026` 已全部通过：architecture、完整 Windows automated acceptance 与 validation evidence upload 均为 SUCCESS；此前 AT1 hard gate run `31295528438` 已通过 Research Service 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。最终 artifact `9033265259` 大小 `14252405` 字节，SHA-256 `730bc98ba24dca2955c7100975ed79fd10a66919fd74a713ec061bddd3b44b59`。AT1 施工和 verifier 修复临时 workflow / 脚本均已清理；R3-07 继续为 `IN_PROGRESS`，OpenAI Research、Fact Pipeline、P4 Research worker 与人工冲突裁决留给后续 Atomic Tasks。'
if anchor not in root:
    raise RuntimeError("root README AT1 closeout anchor missing")
at2 = f'''- Atomic Task 2 迁移 Fact Pipeline：旧 `crates/application/src/fact_pipeline.rs` 删除，`process_p4_research_evidence` 经 ResearchService / `use_cases/research/fact_pipeline/`；实现按协调器、实体解析、时间审计、来源策略、证据持久化、冲突裁决、路由、验证与共享类型拆分，不保留单文件巨型 owner。`FactPipelinePort` 按真实持久化能力收敛并由 `composition/adapters/research.rs` 适配；冲突事件通过序列化新类型穿越 Port，Ports 不暴露裸 JSON Value。OpenAI Gateway、P4 Research worker 与人工冲突裁决仍留给后续 Atomic Tasks。\n- Atomic Task 2 Windows hard gate run `{RUN_ID}` 只有在 Research 专项、Database/Prediction 兼容验证、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests 全部通过后才形成 clean 提交。'''
if "Atomic Task 2 迁移 Fact Pipeline" not in root:
    root = root.replace(anchor, anchor + "\n" + at2, 1)
root_path.write_text(root, encoding="utf-8", newline="\n")

index_path = ROOT / "docs/modular-rewrite/R03-application-services/README.md"
index = index_path.read_text(encoding="utf-8")
index_anchor = '- Atomic Task 1 已正式关闭为 `DONE`。正式 HEAD `ad53bd6c8cdbd93c9e58a38642ae4d22c7d32df7` 的 Public Platform CI run `31296372108` / Windows Automated job `93201993026` 已通过 architecture、完整 Windows automated acceptance 与 evidence upload；AT1 hard gate run `31295528438` 已通过 Research 专项、Ports、architecture、Application check/tests、workspace Clippy/tests。artifact `9033265259` 大小 `14252405` 字节，SHA-256 `730bc98ba24dca2955c7100975ed79fd10a66919fd74a713ec061bddd3b44b59`。临时施工与 verifier 修复 workflow / 脚本均已清理；R3-07 保持 `IN_PROGRESS`，下一 Atomic Task 尚未开始。'
if index_anchor not in index:
    raise RuntimeError("R03 AT1 closeout anchor missing")
index_at2 = f'''- Atomic Task 2 迁移 Fact Pipeline：删除旧 `fact_pipeline.rs`，公共 evidence processing 进入 ResearchService；Fact Pipeline 按职责拆为协调、实体、时间、来源、证据、冲突、路由、验证和共享类型模块，持久化只经 FactPipeline / Evidence Ledger / Artifact Ports。OpenAI Gateway、Research worker 与 conflict mutation 未提前迁移。\n- AT2 Windows hard gate run `{RUN_ID}` 负责 Research/Database/Prediction 专项、Ports、architecture、Application check/tests、workspace Clippy/tests；通过前 R3-07 保持 `IN_PROGRESS`。'''
if "Atomic Task 2 迁移 Fact Pipeline" not in index:
    index = index.replace(index_anchor, index_anchor + "\n" + index_at2, 1)
index_path.write_text(index, encoding="utf-8", newline="\n")

print("R3-07 Atomic Task 2 Fact Pipeline migration generated")
