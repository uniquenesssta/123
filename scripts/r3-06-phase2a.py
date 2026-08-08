from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
ORCH = ROOT / "crates/application/src/p4_orchestration.rs"
WORKBENCH = ROOT / "crates/application/src/p4_workbench.rs"


def match_brace(text: str, open_index: int) -> int:
    depth = 0
    in_string = False
    escaped = False
    in_line_comment = False
    block_depth = 0
    i = open_index
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
            i += 1
            continue
        if block_depth:
            if ch == "/" and nxt == "*":
                block_depth += 1
                i += 2
                continue
            if ch == "*" and nxt == "/":
                block_depth -= 1
                i += 2
                continue
            i += 1
            continue
        if in_string:
            if escaped:
                escaped = False
            elif ch == "\\":
                escaped = True
            elif ch == '"':
                in_string = False
            i += 1
            continue
        if ch == "/" and nxt == "/":
            in_line_comment = True
            i += 2
            continue
        if ch == "/" and nxt == "*":
            block_depth = 1
            i += 2
            continue
        if ch == '"':
            in_string = True
            i += 1
            continue
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise RuntimeError("unmatched brace")


def method_span(text: str, name: str):
    match = re.search(
        rf"(?m)^    pub(?:\([^)]*\))?\s+(?:async\s+)?fn\s+{re.escape(name)}\b",
        text,
    )
    if not match:
        raise RuntimeError(f"method not found: {name}")
    open_index = text.find("{", match.start())
    close_index = match_brace(text, open_index)
    end = close_index + 1
    while end < len(text) and text[end] in " \t":
        end += 1
    if end < len(text) and text[end] == "\r":
        end += 1
    if end < len(text) and text[end] == "\n":
        end += 1
    return match.start(), end, open_index, close_index


def method_body(text: str, name: str) -> str:
    _, _, open_index, close_index = method_span(text, name)
    return text[open_index + 1 : close_index]


def remove_methods(text: str, names):
    spans = [method_span(text, name)[:2] for name in names]
    for start, end in sorted(spans, reverse=True):
        text = text[:start] + text[end:]
    return text


def write(rel: str, content: str):
    path = ROOT / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content.rstrip() + "\n", encoding="utf-8", newline="\n")


orch = ORCH.read_text(encoding="utf-8")
workbench = WORKBENCH.read_text(encoding="utf-8")

# Planning helper owner: shared by the migrated planner and the residual worker.
write(
    "crates/application/src/use_cases/prediction/shared/p4_planning.rs",
    r'''use crate::model_shell::P4_MODEL_ID;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{P4FreezeTaskRecord, P4Horizon, RouteDecision};
use std::collections::BTreeSet;
use uuid::Uuid;

pub(crate) fn validate_requested_fact_keys(
    requested: Vec<String>,
) -> ApplicationResult<Vec<String>> {
    let canonical = canonical_fact_keys();
    if requested.is_empty() {
        return Ok(canonical);
    }
    let requested_set = requested
        .into_iter()
        .map(|value| value.trim().to_string())
        .collect::<BTreeSet<_>>();
    let canonical_set = canonical.iter().cloned().collect::<BTreeSet<_>>();
    if requested_set != canonical_set {
        return Err(ApplicationError::Validation(
            "正式P4冻结必须研究路由注册表中的全部29个事实字段；不得以子集生成31字段正式快照"
                .to_string(),
        ));
    }
    Ok(canonical)
}

pub(crate) fn canonical_fact_keys() -> Vec<String> {
    let registry: football_domain::EvidenceRouteRegistry = serde_json::from_str(include_str!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src-tauri/resources/research/public_evidence_routes.json"
        )
    ))
    .expect("内置P4证据路由注册表必须有效");
    registry
        .routes
        .into_iter()
        .map(|route| route.field_key)
        .collect()
}

pub(crate) fn is_p4_model(model_id: &str) -> bool {
    model_id == P4_MODEL_ID || model_id.starts_with("p4_")
}

pub(crate) fn horizon_priority(horizon: P4Horizon) -> i32 {
    match horizon {
        P4Horizon::T24h => 10,
        P4Horizon::T6h => 20,
        P4Horizon::T90m => 30,
        P4Horizon::T1h => 40,
        P4Horizon::LegacyTN => 0,
    }
}

pub(crate) fn validate_existing_task_identity(
    task: &P4FreezeTaskRecord,
    route: &RouteDecision,
    research_schema_version_id: Uuid,
    snapshot_schema_version_id: Uuid,
    requested_fact_keys: &[String],
) -> ApplicationResult<()> {
    let existing_facts = task
        .requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let requested_facts = requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if task.rule_package_id != route.rule_package_id
        || task.model_version_id != route.model_version_id
        || task.parameter_set_id != route.parameter_set_id
        || task.competition_profile_id != route.competition_profile_id
        || task.research_schema_version_id != research_schema_version_id
        || task.snapshot_schema_version_id != snapshot_schema_version_id
        || existing_facts != requested_facts
    {
        return Err(ApplicationError::Validation(
            "同一正式队列键已存在，但规则包、模型、参数、赛事Profile、Schema或事实字段与当前规划请求不一致"
                .to_string(),
        ));
    }
    Ok(())
}
''',
)
shared_mod = ROOT / "crates/application/src/use_cases/prediction/shared/mod.rs"
shared = shared_mod.read_text(encoding="utf-8")
if "pub(crate) mod p4_planning;" not in shared:
    shared = shared.rstrip() + "\npub(crate) mod p4_planning;\n"
shared_mod.write_text(shared, encoding="utf-8", newline="\n")

# Migrate plan_p4_horizons through existing Ports.
plan = method_body(orch, "plan_p4_horizons")
plan = plan.replace("let store = self.active_store().await?;", "let store = port;")
plan = plan.replace(".p4_planning_match_context(", ".planning_match_context(")
plan = plan.replace(".read_schema_version_by_key(", ".read_schema(")
plan = plan.replace(".find_p4_freeze_task_by_idempotency(", ".find_freeze_task_by_idempotency(")
plan = plan.replace(".create_p4_freeze_task(", ".create_freeze_task(")
plan = plan.replace(".enqueue_job(", ".enqueue(")
plan = plan.replace(".transition_p4_freeze_task(&P4FreezeTaskTransition {", ".transition_freeze_task(task.id, &P4FreezeTaskTransition {")
write(
    "crates/application/src/use_cases/prediction/plan_p4_horizons/mod.rs",
    f'''use super::P4PlanningAccess;
use super::shared::p4_planning::{{
    horizon_priority, is_p4_model, validate_existing_task_identity, validate_requested_fact_keys,
}};
use crate::built_in_artifacts::{{
    P4_RESEARCH_SCHEMA_ARTIFACT_VERSION as RESEARCH_SCHEMA_VERSION,
    P4_RESEARCH_SCHEMA_KEY as RESEARCH_SCHEMA_KEY,
    P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION as SNAPSHOT_SCHEMA_VERSION,
    P4_SNAPSHOT_SCHEMA_KEY as SNAPSHOT_SCHEMA_KEY,
}};
use crate::{{ApplicationError, ApplicationResult}};
use chrono::{{Duration, Utc}};
use football_domain::{{
    EnqueueJobDraft, P4FreezeTaskDraft, P4FreezeTaskRecord, P4FreezeTaskState,
    P4FreezeTaskTransition, P4Horizon, PlanP4HorizonsCommand, RouteRequest,
    P4_FREEZE_GRACE_MINUTES, P4_ORCHESTRATION_PLANNER_VERSION, P4_RESEARCH_LEAD_MINUTES,
}};
use serde_json::{{json, Value}};
use uuid::Uuid;

const P4_RESEARCH_JOB: &str = "p4_horizon_research";

pub(crate) async fn execute<P: P4PlanningAccess + ?Sized>(
    port: &P,
    command: PlanP4HorizonsCommand,
) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {{{plan}
}}
''',
)

# Simple workflow/read-only use cases.
use_cases = {
    "list_p4_freeze_tasks": '''use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskRecord;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    match_id: Option<Uuid>,
    limit: u32,
) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
    Ok(port.list_freeze_tasks(match_id, limit).await?)
}
''',
    "read_p4_freeze_task": '''use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskRecord;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    task_id: Uuid,
) -> ApplicationResult<P4FreezeTaskRecord> {
    Ok(port.read_freeze_task(task_id).await?)
}
''',
    "list_p4_freeze_task_events": '''use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskEventRecord;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    task_id: Uuid,
) -> ApplicationResult<Vec<P4FreezeTaskEventRecord>> {
    Ok(port.list_freeze_task_events(task_id).await?)
}
''',
    "p4_freeze_readiness": '''use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeReadiness;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    task_id: Uuid,
) -> ApplicationResult<P4FreezeReadiness> {
    Ok(port.freeze_readiness(task_id).await?)
}
''',
    "read_p4_match_workspace": '''use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4MatchWorkspace;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    match_id: Uuid,
) -> ApplicationResult<P4MatchWorkspace> {
    Ok(port.read_match_workspace(match_id).await?)
}
''',
    "read_p4_task_workspace": '''use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4TaskWorkspace;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    task_id: Uuid,
) -> ApplicationResult<P4TaskWorkspace> {
    Ok(port.read_task_workspace(task_id).await?)
}
''',
}
for name, content in use_cases.items():
    write(f"crates/application/src/use_cases/prediction/{name}/mod.rs", content)

# Expand use-case registry and capability aggregate.
mod_path = ROOT / "crates/application/src/use_cases/prediction/mod.rs"
mod_text = mod_path.read_text(encoding="utf-8")
for name in [
    "list_p4_freeze_task_events",
    "list_p4_freeze_tasks",
    "p4_freeze_readiness",
    "plan_p4_horizons",
    "read_p4_freeze_task",
    "read_p4_match_workspace",
    "read_p4_task_workspace",
]:
    line = f"pub(crate) mod {name};"
    if line not in mod_text:
        mod_text = mod_text.replace("pub(crate) mod shared;", f"pub(crate) mod {name};\npub(crate) mod shared;")
if "P4PlanningAccess" not in mod_text:
    mod_text += '''

pub(crate) trait P4PlanningAccess:
    crate::ports::prediction::PredictionWorkflowPort
    + crate::ports::rules::RuleRoutingPort
    + crate::ports::research::ResearchArtifactPort
    + crate::ports::analytics::JobQueuePort
{
}

impl<T> P4PlanningAccess for T where
    T: crate::ports::prediction::PredictionWorkflowPort
        + crate::ports::rules::RuleRoutingPort
        + crate::ports::research::ResearchArtifactPort
        + crate::ports::analytics::JobQueuePort
{
}
'''
mod_path.write_text(mod_text, encoding="utf-8", newline="\n")

# Extend PredictionWorkflowPort with the actual Prediction-owned P4 read/planning surface.
ports_path = ROOT / "crates/application/src/ports/prediction/mod.rs"
ports = ports_path.read_text(encoding="utf-8")
ports = ports.replace(
    "    P4FreezeTaskTransition, PredictionSummary, PreparedMatchPredictionInput, RouteDecision,\n",
    "    P4FreezeTaskTransition, P4MatchWorkspace, P4PlanningMatchContext, P4TaskWorkspace,\n    PredictionSummary, PreparedMatchPredictionInput, RouteDecision,\n",
)
marker = "pub trait PredictionWorkflowPort: Send + Sync {\n"
insert = '''pub trait PredictionWorkflowPort: Send + Sync {
    async fn planning_match_context(&self, match_id: Uuid) -> PortResult<P4PlanningMatchContext>;
    async fn find_freeze_task_by_idempotency(
        &self,
        idempotency_key: &str,
    ) -> PortResult<Option<P4FreezeTaskRecord>>;
    async fn list_freeze_tasks(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PortResult<Vec<P4FreezeTaskRecord>>;
    async fn read_match_workspace(&self, match_id: Uuid) -> PortResult<P4MatchWorkspace>;
    async fn read_task_workspace(&self, task_id: Uuid) -> PortResult<P4TaskWorkspace>;
'''
if marker not in ports:
    raise RuntimeError("PredictionWorkflowPort marker missing")
ports = ports.replace(marker, insert, 1)
ports_path.write_text(ports, encoding="utf-8", newline="\n")

# ResearchArtifactPort needs the existing immutable schema lookup used by the planner.
research_path = ROOT / "crates/application/src/ports/research/mod.rs"
research = research_path.read_text(encoding="utf-8")
needle = "pub trait ResearchArtifactPort: Send + Sync {\n"
if "async fn read_schema(" not in research:
    research = research.replace(
        needle,
        needle + "    async fn read_schema(&self, schema_key: &str, version: &str) -> PortResult<SchemaVersionRecord>;\n",
        1,
    )
research_path.write_text(research, encoding="utf-8", newline="\n")

# Prediction workflow adapter.
adapter_path = ROOT / "crates/application/src/composition/adapters/prediction.rs"
adapter = adapter_path.read_text(encoding="utf-8")
adapter = adapter.replace(
    "    prediction::{ModelRunHistoryItem, ModelRunPort, PredictionInputPort, SerializedModelRun},\n",
    "    prediction::{\n        ModelRunHistoryItem, ModelRunPort, PredictionInputPort, PredictionWorkflowPort,\n        SerializedModelRun,\n    },\n",
)
adapter = adapter.replace(
    "use football_domain::{PredictionSummary, PreparedMatchPredictionInput, RouteDecision};",
    "use football_domain::{\n    P4FreezeReadiness, P4FreezeTaskDraft, P4FreezeTaskEventRecord, P4FreezeTaskRecord,\n    P4FreezeTaskTransition, P4MatchWorkspace, P4PlanningMatchContext, P4TaskWorkspace,\n    PredictionSummary, PreparedMatchPredictionInput, RouteDecision,\n};",
)
if "impl PredictionWorkflowPort for ActiveDatabase" not in adapter:
    adapter += '''

#[async_trait]
impl PredictionWorkflowPort for ActiveDatabase {
    async fn planning_match_context(&self, match_id: Uuid) -> PortResult<P4PlanningMatchContext> {
        self.transition_store()
            .p4_planning_match_context(match_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn find_freeze_task_by_idempotency(
        &self,
        idempotency_key: &str,
    ) -> PortResult<Option<P4FreezeTaskRecord>> {
        self.transition_store()
            .find_p4_freeze_task_by_idempotency(idempotency_key)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_freeze_tasks(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PortResult<Vec<P4FreezeTaskRecord>> {
        self.transition_store()
            .list_p4_freeze_tasks(match_id, limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_freeze_task(
        &self,
        draft: &P4FreezeTaskDraft,
    ) -> PortResult<P4FreezeTaskRecord> {
        self.transition_store()
            .create_p4_freeze_task(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_freeze_task(&self, task_id: Uuid) -> PortResult<P4FreezeTaskRecord> {
        self.transition_store()
            .read_p4_freeze_task(task_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_freeze_task_events(
        &self,
        task_id: Uuid,
    ) -> PortResult<Vec<P4FreezeTaskEventRecord>> {
        self.transition_store()
            .list_p4_freeze_task_events(task_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn transition_freeze_task(
        &self,
        task_id: Uuid,
        transition: &P4FreezeTaskTransition,
    ) -> PortResult<P4FreezeTaskRecord> {
        if transition.task_id != task_id {
            return Err(PortError::new(
                PortErrorKind::InvalidState,
                "P4冻结任务迁移的task_id与transition不一致",
            ));
        }
        self.transition_store()
            .transition_p4_freeze_task(transition)
            .await
            .map_err(map_persistence_error)
    }

    async fn freeze_readiness(&self, task_id: Uuid) -> PortResult<P4FreezeReadiness> {
        self.transition_store()
            .p4_freeze_readiness(task_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_match_workspace(&self, match_id: Uuid) -> PortResult<P4MatchWorkspace> {
        self.transition_store()
            .read_p4_match_workspace(match_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_task_workspace(&self, task_id: Uuid) -> PortResult<P4TaskWorkspace> {
        self.transition_store()
            .read_p4_task_workspace(task_id)
            .await
            .map_err(map_persistence_error)
    }
}
'''
adapter_path.write_text(adapter, encoding="utf-8", newline="\n")

# Research artifact adapter (only composition changes; Research service remains R3-07).
write(
    "crates/application/src/composition/adapters/research.rs",
    '''use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{research::ResearchArtifactPort, PortResult};
use async_trait::async_trait;
use football_domain::{
    PromptVersionDraft, PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft,
    ResearchRunRecord, SchemaVersionDraft, SchemaVersionRecord, SourcePolicyVersionDraft,
    SourcePolicyVersionRecord,
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

    async fn record_run_event(&self, draft: &ResearchRunEventDraft) -> PortResult<()> {
        self.transition_store()
            .record_research_run_event(draft)
            .await
            .map(|_| ())
            .map_err(map_persistence_error)
    }
}
''',
)

# Existing JobQueuePort concrete adapter required by planning; analytics ownership is not migrated.
write(
    "crates/application/src/composition/adapters/jobs.rs",
    '''use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{analytics::JobQueuePort, PortResult};
use async_trait::async_trait;
use football_domain::{BackgroundJob, EnqueueJobDraft};
use uuid::Uuid;

#[async_trait]
impl JobQueuePort for ActiveDatabase {
    async fn enqueue(&self, draft: &EnqueueJobDraft) -> PortResult<BackgroundJob> {
        self.transition_store()
            .enqueue_job(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_jobs(&self, limit: i64) -> PortResult<Vec<BackgroundJob>> {
        let limit = u32::try_from(limit.clamp(1, 500)).unwrap_or(500);
        self.transition_store()
            .list_jobs(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn request_cancellation(&self, job_id: Uuid) -> PortResult<BackgroundJob> {
        self.transition_store()
            .request_job_cancellation(job_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn retry(&self, job_id: Uuid) -> PortResult<BackgroundJob> {
        self.transition_store()
            .retry_job(job_id)
            .await
            .map_err(map_persistence_error)
    }
}
''',
)
adapters_mod = ROOT / "crates/application/src/composition/adapters/mod.rs"
adapters = adapters_mod.read_text(encoding="utf-8")
for module in ["jobs", "research"]:
    line = f"mod {module};"
    if line not in adapters:
        adapters = adapters.replace("mod lineups;", f"mod lineups;\n{line}")
adapters_mod.write_text(adapters, encoding="utf-8", newline="\n")

# PredictionService owns the migrated P4 planning/read operations.
service_path = ROOT / "crates/application/src/services/prediction/service.rs"
service = service_path.read_text(encoding="utf-8")
service = service.replace(
    "    hide_run_from_history, inspect_match_prediction_readiness, list_recent_runs, preview_route,\n    read_run, PredictionAccess,\n",
    "    hide_run_from_history, inspect_match_prediction_readiness, list_p4_freeze_task_events,\n    list_p4_freeze_tasks, list_recent_runs, p4_freeze_readiness, plan_p4_horizons, preview_route,\n    read_p4_freeze_task, read_p4_match_workspace, read_p4_task_workspace, read_run,\n    P4PlanningAccess, PredictionAccess,\n",
)
service = service.replace(
    "use football_domain::{MatchPredictionReadiness, RouteDecision};",
    "use football_domain::{\n    MatchPredictionReadiness, P4FreezeReadiness, P4FreezeTaskEventRecord, P4FreezeTaskRecord,\n    P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, RouteDecision,\n};",
)
insert_before = "    pub(crate) async fn read_run<P: PredictionAccess + ?Sized>("
extra = '''    pub(crate) async fn plan_p4_horizons<P: P4PlanningAccess + ?Sized>(
        &self,
        port: &P,
        command: PlanP4HorizonsCommand,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        plan_p4_horizons::execute(port, command).await
    }

    pub(crate) async fn list_p4_freeze_tasks<P: crate::ports::prediction::PredictionWorkflowPort + ?Sized>(
        &self,
        port: &P,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        list_p4_freeze_tasks::execute(port, match_id, limit).await
    }

    pub(crate) async fn read_p4_freeze_task<P: crate::ports::prediction::PredictionWorkflowPort + ?Sized>(
        &self,
        port: &P,
        task_id: Uuid,
    ) -> ApplicationResult<P4FreezeTaskRecord> {
        read_p4_freeze_task::execute(port, task_id).await
    }

    pub(crate) async fn list_p4_freeze_task_events<P: crate::ports::prediction::PredictionWorkflowPort + ?Sized>(
        &self,
        port: &P,
        task_id: Uuid,
    ) -> ApplicationResult<Vec<P4FreezeTaskEventRecord>> {
        list_p4_freeze_task_events::execute(port, task_id).await
    }

    pub(crate) async fn p4_freeze_readiness<P: crate::ports::prediction::PredictionWorkflowPort + ?Sized>(
        &self,
        port: &P,
        task_id: Uuid,
    ) -> ApplicationResult<P4FreezeReadiness> {
        p4_freeze_readiness::execute(port, task_id).await
    }

    pub(crate) async fn read_p4_match_workspace<P: crate::ports::prediction::PredictionWorkflowPort + ?Sized>(
        &self,
        port: &P,
        match_id: Uuid,
    ) -> ApplicationResult<P4MatchWorkspace> {
        read_p4_match_workspace::execute(port, match_id).await
    }

    pub(crate) async fn read_p4_task_workspace<P: crate::ports::prediction::PredictionWorkflowPort + ?Sized>(
        &self,
        port: &P,
        task_id: Uuid,
    ) -> ApplicationResult<P4TaskWorkspace> {
        read_p4_task_workspace::execute(port, task_id).await
    }

'''
if extra.strip() not in service:
    service = service.replace(insert_before, extra + insert_before, 1)
service_path.write_text(service, encoding="utf-8", newline="\n")

# Public compatibility facade: names/signatures unchanged.
facade_path = ROOT / "crates/application/src/services/prediction/facade.rs"
facade = facade_path.read_text(encoding="utf-8")
facade = facade.replace(
    "use football_domain::{MatchPredictionReadiness, RouteDecision};",
    "use football_domain::{\n    MatchPredictionReadiness, P4FreezeReadiness, P4FreezeTaskEventRecord, P4FreezeTaskRecord,\n    P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, RouteDecision,\n};",
)
closing = facade.rfind("}\n")
if closing < 0:
    raise RuntimeError("prediction facade closing brace missing")
methods = '''
    pub async fn plan_p4_horizons(
        &self,
        command: PlanP4HorizonsCommand,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        let session = self.prediction_session().await?;
        self.prediction.plan_p4_horizons(&session, command).await
    }

    pub async fn list_p4_freeze_tasks(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        let session = self.prediction_session().await?;
        self.prediction
            .list_p4_freeze_tasks(&session, match_id, limit)
            .await
    }

    pub async fn read_p4_freeze_task(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<P4FreezeTaskRecord> {
        let session = self.prediction_session().await?;
        self.prediction.read_p4_freeze_task(&session, task_id).await
    }

    pub async fn list_p4_freeze_task_events(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<Vec<P4FreezeTaskEventRecord>> {
        let session = self.prediction_session().await?;
        self.prediction
            .list_p4_freeze_task_events(&session, task_id)
            .await
    }

    pub async fn p4_freeze_readiness(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<P4FreezeReadiness> {
        let session = self.prediction_session().await?;
        self.prediction.p4_freeze_readiness(&session, task_id).await
    }

    pub async fn read_p4_match_workspace(
        &self,
        match_id: Uuid,
    ) -> ApplicationResult<P4MatchWorkspace> {
        let session = self.prediction_session().await?;
        self.prediction
            .read_p4_match_workspace(&session, match_id)
            .await
    }

    pub async fn read_p4_task_workspace(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<P4TaskWorkspace> {
        let session = self.prediction_session().await?;
        self.prediction
            .read_p4_task_workspace(&session, task_id)
            .await
    }
'''
if "pub async fn plan_p4_horizons(" not in facade:
    facade = facade[:closing] + methods + facade[closing:]
facade_path.write_text(facade, encoding="utf-8", newline="\n")

# Remove migrated methods from legacy mixed owners.
orch = remove_methods(
    orch,
    [
        "plan_p4_horizons",
        "list_p4_freeze_tasks",
        "read_p4_freeze_task",
        "list_p4_freeze_task_events",
        "p4_freeze_readiness",
    ],
)
# Remove helper definitions now owned by shared Prediction planning and import them for residual worker/tests.
for helper in [
    "validate_requested_fact_keys",
    "canonical_fact_keys",
    "is_p4_model",
    "horizon_priority",
    "validate_existing_task_identity",
]:
    match = re.search(rf"(?m)^fn\s+{helper}\b", orch)
    if match:
        open_index = orch.find("{", match.start())
        close_index = match_brace(orch, open_index)
        end = close_index + 1
        while end < len(orch) and orch[end] in " \t\r\n":
            if orch[end] == "\n" and end + 1 < len(orch) and orch[end + 1] not in "\r\n":
                end += 1
                break
            end += 1
        orch = orch[:match.start()] + orch[end:]
helper_import = "use crate::use_cases::prediction::shared::p4_planning::{canonical_fact_keys, horizon_priority, is_p4_model};\n"
if helper_import not in orch:
    orch = helper_import + orch
ORCH.write_text(orch, encoding="utf-8", newline="\n")

workbench = remove_methods(workbench, ["read_p4_match_workspace", "read_p4_task_workspace"])
WORKBENCH.write_text(workbench, encoding="utf-8", newline="\n")

print("R3-06 phase 2A P4 planning/read migration generated")
