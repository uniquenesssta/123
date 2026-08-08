from pathlib import Path
import re
import textwrap

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/application/src/prediction.rs"
source = SRC.read_text(encoding="utf-8")


def match_brace(text: str, open_index: int) -> int:
    depth = 0
    in_string = False
    escaped = False
    in_line_comment = False
    in_block_comment = 0
    i = open_index
    while i < len(text):
        ch = text[i]
        nxt = text[i + 1] if i + 1 < len(text) else ""
        if in_line_comment:
            if ch == "\n":
                in_line_comment = False
            i += 1
            continue
        if in_block_comment:
            if ch == "/" and nxt == "*":
                in_block_comment += 1
                i += 2
                continue
            if ch == "*" and nxt == "/":
                in_block_comment -= 1
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
            in_block_comment = 1
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


def method_body(name: str) -> str:
    pattern = re.compile(rf"(?m)^    (?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+{re.escape(name)}\b")
    match = pattern.search(source)
    if not match:
        raise RuntimeError(f"method not found: {name}")
    open_index = source.find("{", match.start())
    close_index = match_brace(source, open_index)
    return source[open_index + 1 : close_index]


def free_item(name: str) -> str:
    pattern = re.compile(rf"(?m)^(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+{re.escape(name)}\b")
    match = pattern.search(source)
    if not match:
        raise RuntimeError(f"free function not found: {name}")
    start = match.start()
    open_index = source.find("{", start)
    close_index = match_brace(source, open_index)
    return source[start : close_index + 1]


def make_public(item: str) -> str:
    item = re.sub(r"^(?:pub\([^)]*\)\s+)?fn\s+", "pub(crate) fn ", item, count=1)
    item = re.sub(r"^(?:pub\([^)]*\)\s+)?async\s+fn\s+", "pub(crate) async fn ", item, count=1)
    return item


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content.rstrip() + "\n", encoding="utf-8", newline="\n")


def transform_store(body: str) -> str:
    body = body.replace("let store = self.active_store().await?;", "let store = port;")
    body = body.replace("self.registry", "registry")
    body = body.replace(".prepare_match_prediction_input_at(", ".prepare_match_input_at(")
    body = body.replace(".read_match_lineup_chain_at(", ".read_match_chain_at(")
    body = body.replace("Err(PersistenceError::InvalidState(message)) => {", "Err(error) if error.kind == PortErrorKind::InvalidState => {\n                let message = error.message;")
    body = body.replace("Err(PersistenceError::RouteNotFound) =>", "Err(error) if error.kind == PortErrorKind::NotFound =>")
    return body


# Shared routing helpers
routing_names = [
    "match_context_from_command",
    "ensure_match_input_id",
    "compact_key_part",
    "validate_snapshot_type",
    "route_identity_manifest",
    "verify_route_identity_matches_input_audit",
    "normalize_model_selection",
    "ensure_model_selection_registered",
    "parse_kickoff",
    "required_string",
    "nested_required_string",
]
routing_items = []
for name in routing_names:
    item = make_public(free_item(name))
    item = item.replace("registry: &crate::ModelRegistry", "registry: &ModelRegistry")
    routing_items.append(item)
model_selection = re.search(
    r"#\[derive\(Debug, Clone\)\]\nstruct ModelSelection \{.*?\n\}", source, re.S
)
if not model_selection:
    raise RuntimeError("ModelSelection not found")
model_selection_text = model_selection.group(0).replace(
    "struct ModelSelection {", "pub(crate) struct ModelSelection {"
)
model_selection_text = model_selection_text.replace(
    "    family: &'static str,", "    pub(crate) family: &'static str,"
).replace(
    "    exact_model_id: Option<String>,", "    pub(crate) exact_model_id: Option<String>,"
)
write(
    "crates/application/src/use_cases/prediction/shared/routing.rs",
    """use crate::{model_registry::ModelRegistry, ApplicationError, ApplicationResult, PredictionCommand};
use chrono::{DateTime, Utc};
use football_domain::{MatchContext, ResolvedCompetitionContext, RouteDecision, RuleRouting};
use serde_json::{json, Value};
use uuid::Uuid;

""" + "\n\n".join(routing_items[:6]) + "\n\n" + model_selection_text + "\n\n" + "\n\n".join(routing_items[6:]),
)

# Shared input audit helpers
audit_names = [
    "build_prediction_input_manifest",
    "strip_runtime_prediction_input_identity",
    "verify_prepared_input_matches_readiness",
    "attach_prediction_input_audit",
    "prediction_input_audit_summary",
    "sha256_value",
]
audit_items = [make_public(free_item(name)) for name in audit_names]
write(
    "crates/application/src/use_cases/prediction/shared/audit.rs",
    """use crate::{ApplicationError, ApplicationResult};
use football_domain::{MatchPredictionReadiness, PredictionInputAuditSummary};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

""" + "\n\n".join(audit_items),
)

# Readiness-only checks
readiness_names = [
    "readiness_check",
    "selected_lineup",
    "append_unavailable_lineup_readiness_checks",
    "append_lineup_readiness_checks",
    "append_unavailable_prepared_input_checks",
    "append_prepared_input_checks",
    "nested_u64",
]
readiness_items = [make_public(free_item(name)) for name in readiness_names]
write(
    "crates/application/src/use_cases/prediction/shared/readiness_checks.rs",
    """use football_domain::{
    MatchLineupChain, PredictionReadinessCheck, PredictionReadinessCheckStatus,
};
use serde_json::{json, Value};

""" + "\n\n".join(readiness_items),
)
write(
    "crates/application/src/use_cases/prediction/shared/mod.rs",
    "pub(crate) mod audit;\npub(crate) mod readiness_checks;\npub(crate) mod routing;",
)

# Core execution
execute_internal = transform_store(method_body("execute_prediction_internal"))
execute_internal = execute_internal.replace(
    "ensure_model_selection_registered(registry, &model_selection)?;", "ensure_model_selection_registered(registry, &model_selection)?;"
)
write(
    "crates/application/src/use_cases/prediction/execute_prediction/mod.rs",
    f"""use super::super::PredictionAccess;
use super::super::shared::audit::{{prediction_input_audit_summary, sha256_value}};
use super::super::shared::routing::{{
    ensure_match_input_id, ensure_model_selection_registered, match_context_from_command,
    normalize_model_selection, validate_snapshot_type, verify_route_identity_matches_input_audit,
}};
use crate::model_registry::ModelRegistry;
use crate::{{ApplicationError, ApplicationResult, PredictionCommand, PredictionExecution}};
use football_domain::{{ModelIdentity, RouteRequest}};
use football_model_api::ModelRequest;
use serde_json::json;
use std::time::Instant;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: PredictionCommand,
) -> ApplicationResult<PredictionExecution> {{
    execute_internal(port, registry, command, true).await
}}

pub(crate) async fn execute_internal<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: PredictionCommand,
    persist_run: bool,
) -> ApplicationResult<PredictionExecution> {{{execute_internal}
}}
""",
)

# Readiness assessment
readiness_body = transform_store(method_body("inspect_match_prediction_readiness"))
write(
    "crates/application/src/use_cases/prediction/inspect_match_prediction_readiness/mod.rs",
    f"""use super::super::PredictionAccess;
use super::super::shared::audit::{{build_prediction_input_manifest, sha256_value}};
use super::super::shared::readiness_checks::{{
    append_lineup_readiness_checks, append_prepared_input_checks,
    append_unavailable_lineup_readiness_checks, append_unavailable_prepared_input_checks,
    readiness_check,
}};
use super::super::shared::routing::{{
    ensure_model_selection_registered, normalize_model_selection, route_identity_manifest,
}};
use crate::model_registry::ModelRegistry;
use crate::ports::PortErrorKind;
use crate::{{ApplicationError, ApplicationResult, StoredMatchPredictionCommand}};
use chrono::Utc;
use football_domain::{{
    MatchPredictionReadiness, PredictionReadinessCheck, PredictionReadinessCheckStatus,
    PredictionReadinessLevel, RouteRequest, PREDICTION_INPUT_AUDIT_VERSION,
}};
use serde_json::{{json, Value}};

pub(crate) async fn execute<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
) -> ApplicationResult<MatchPredictionReadiness> {{{readiness_body}
}}
""",
)

# Stored-match formal/shadow execution
stored_body = transform_store(method_body("execute_prediction_from_match_with_mode"))
stored_body = stored_body.replace(
    "self.inspect_match_prediction_readiness(command.clone()).await?",
    "super::inspect_match_prediction_readiness::execute(port, registry, command.clone()).await?",
)
stored_body = stored_body.replace(
    "self.execute_prediction_internal(",
    "super::execute_prediction::execute_internal(port, registry, ",
)
write(
    "crates/application/src/use_cases/prediction/execute_prediction_from_match/mod.rs",
    f"""use super::super::PredictionAccess;
use super::super::shared::audit::{{attach_prediction_input_audit, verify_prepared_input_matches_readiness}};
use crate::model_registry::ModelRegistry;
use crate::{{
    ApplicationError, ApplicationResult, PredictionCommand, PredictionExecution,
    StoredMatchPredictionCommand,
}};

pub(crate) async fn execute_formal<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
) -> ApplicationResult<PredictionExecution> {{
    execute_with_mode(port, registry, command, false).await
}}

pub(crate) async fn execute_shadow<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
) -> ApplicationResult<PredictionExecution> {{
    execute_with_mode(port, registry, command, true).await
}}

async fn execute_with_mode<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
    shadow: bool,
) -> ApplicationResult<PredictionExecution> {{{stored_body}
}}
""",
)

# Route preview
preview_body = transform_store(method_body("preview_route"))
write(
    "crates/application/src/use_cases/prediction/preview_route/mod.rs",
    f"""use super::super::PredictionAccess;
use super::super::shared::routing::{{
    ensure_model_selection_registered, match_context_from_command, normalize_model_selection,
}};
use crate::model_registry::ModelRegistry;
use crate::{{ApplicationResult, RoutePreviewCommand}};
use football_domain::{{RouteDecision, RouteRequest}};

pub(crate) async fn execute<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: RoutePreviewCommand,
) -> ApplicationResult<RouteDecision> {{{preview_body}
}}
""",
)

# Dry run remains model-registry only
dry_body = method_body("dry_run_default_fixture").replace("self.registry", "registry")
write(
    "crates/application/src/use_cases/prediction/dry_run_default_fixture/mod.rs",
    f"""use crate::model_registry::ModelRegistry;
use crate::model_shell::P4_MODEL_ID;
use crate::{{p4_default_match, p4_default_parameters, ApplicationError, ApplicationResult}};
use football_domain::{{ModelIdentity}};
use football_model_api::{{ModelOutput, ModelRequest}};

pub(crate) fn execute(registry: &ModelRegistry) -> ApplicationResult<ModelOutput> {{{dry_body}
}}
""",
)

# History use cases
write(
    "crates/application/src/use_cases/prediction/list_recent_runs/mod.rs",
    """use crate::ports::prediction::{ModelRunHistoryItem, ModelRunPort};
use crate::ApplicationResult;

pub(crate) async fn execute<P: ModelRunPort + ?Sized>(
    port: &P,
    limit: i64,
) -> ApplicationResult<Vec<ModelRunHistoryItem>> {
    Ok(port.list_recent_runs(limit).await?)
}
""",
)
write(
    "crates/application/src/use_cases/prediction/hide_run_from_history/mod.rs",
    """use crate::ports::prediction::ModelRunPort;
use crate::ApplicationResult;
use uuid::Uuid;

pub(crate) async fn execute<P: ModelRunPort + ?Sized>(
    port: &P,
    run_id: Uuid,
    reason: Option<String>,
) -> ApplicationResult<()> {
    Ok(port.hide_run_from_history(run_id, reason.as_deref()).await?)
}
""",
)
write(
    "crates/application/src/use_cases/prediction/read_run/mod.rs",
    """use crate::ports::prediction::ModelRunPort;
use crate::ApplicationResult;
use serde_json::Value;
use uuid::Uuid;

pub(crate) async fn execute<P: ModelRunPort + ?Sized>(
    port: &P,
    run_id: Uuid,
) -> ApplicationResult<Value> {
    let document = port.read_run_document(run_id).await?;
    Ok(serde_json::from_str(&document.json)?)
}
""",
)

write(
    "crates/application/src/use_cases/prediction/mod.rs",
    """use crate::ports::{
    lineup::{LineupPort, MatchCatalogPort},
    prediction::{ModelRunPort, PredictionInputPort},
    rules::RuleRoutingPort,
};

pub(crate) mod dry_run_default_fixture;
pub(crate) mod execute_prediction;
pub(crate) mod execute_prediction_from_match;
pub(crate) mod hide_run_from_history;
pub(crate) mod inspect_match_prediction_readiness;
pub(crate) mod list_recent_runs;
pub(crate) mod preview_route;
pub(crate) mod read_run;
pub(crate) mod shared;

pub(crate) trait PredictionAccess:
    PredictionInputPort + ModelRunPort + RuleRoutingPort + MatchCatalogPort + LineupPort
{
}

impl<T> PredictionAccess for T where
    T: PredictionInputPort + ModelRunPort + RuleRoutingPort + MatchCatalogPort + LineupPort
{
}

#[cfg(test)]
mod tests;
""",
)

# Preserve the existing prediction unit tests under the new module owner.
test_start = source.index("#[cfg(test)]\nmod tests {")
test_open = source.index("{", test_start)
test_close = match_brace(source, test_open)
test_body = source[test_open + 1 : test_close]
test_body = test_body.replace(
    "    use super::*;",
    """    use super::shared::audit::{build_prediction_input_manifest, prediction_input_audit_summary, sha256_value};
    use super::shared::routing::{ensure_model_selection_registered, normalize_model_selection};
    use crate::{ApplicationError, ApplicationService};
    use chrono::Utc;
    use football_domain::PREDICTION_INPUT_AUDIT_VERSION;
    use serde_json::json;
    use uuid::Uuid;""",
)
write("crates/application/src/use_cases/prediction/tests.rs", test_body)

# Prediction service delegates one responsibility per use case.
write(
    "crates/application/src/services/prediction/service.rs",
    """use crate::model_registry::ModelRegistry;
use crate::ports::prediction::ModelRunHistoryItem;
use crate::use_cases::prediction::{
    dry_run_default_fixture, execute_prediction, execute_prediction_from_match,
    hide_run_from_history, inspect_match_prediction_readiness, list_recent_runs, preview_route,
    read_run, PredictionAccess,
};
use crate::{
    ApplicationResult, PredictionCommand, PredictionExecution, RoutePreviewCommand,
    StoredMatchPredictionCommand,
};
use football_domain::{MatchPredictionReadiness, RouteDecision};
use football_model_api::ModelOutput;
use serde_json::Value;
use uuid::Uuid;

pub(crate) struct PredictionService;

impl PredictionService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn execute_prediction<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: PredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        execute_prediction::execute(port, registry, command).await
    }

    pub(crate) async fn inspect_match_prediction_readiness<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<MatchPredictionReadiness> {
        inspect_match_prediction_readiness::execute(port, registry, command).await
    }

    pub(crate) async fn execute_prediction_from_match<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        execute_prediction_from_match::execute_formal(port, registry, command).await
    }

    pub(crate) async fn execute_shadow_prediction_from_match<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        execute_prediction_from_match::execute_shadow(port, registry, command).await
    }

    pub(crate) async fn preview_route<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: RoutePreviewCommand,
    ) -> ApplicationResult<RouteDecision> {
        preview_route::execute(port, registry, command).await
    }

    pub(crate) fn dry_run_default_fixture(
        &self,
        registry: &ModelRegistry,
    ) -> ApplicationResult<ModelOutput> {
        dry_run_default_fixture::execute(registry)
    }

    pub(crate) async fn list_recent_runs<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        limit: i64,
    ) -> ApplicationResult<Vec<ModelRunHistoryItem>> {
        list_recent_runs::execute(port, limit).await
    }

    pub(crate) async fn hide_run_from_history<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        run_id: Uuid,
        reason: Option<String>,
    ) -> ApplicationResult<()> {
        hide_run_from_history::execute(port, run_id, reason).await
    }

    pub(crate) async fn read_run<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        run_id: Uuid,
    ) -> ApplicationResult<Value> {
        read_run::execute(port, run_id).await
    }
}
""",
)

write(
    "crates/application/src/services/prediction/facade.rs",
    """use crate::composition::{model_run_list_item_from_port, ActiveDatabase};
use crate::{
    ApplicationError, ApplicationResult, ApplicationService, ModelRunListItem, PredictionCommand,
    PredictionExecution, RoutePreviewCommand, StoredMatchPredictionCommand,
};
use football_domain::{MatchPredictionReadiness, RouteDecision};
use football_model_api::ModelOutput;
use serde_json::Value;
use uuid::Uuid;

impl ApplicationService {
    async fn prediction_session(&self) -> ApplicationResult<ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    pub async fn execute_prediction(
        &self,
        command: PredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        let session = self.prediction_session().await?;
        self.prediction
            .execute_prediction(&session, &self.registry, command)
            .await
    }

    pub async fn inspect_match_prediction_readiness(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<MatchPredictionReadiness> {
        let session = self.prediction_session().await?;
        self.prediction
            .inspect_match_prediction_readiness(&session, &self.registry, command)
            .await
    }

    pub async fn execute_prediction_from_match(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        let session = self.prediction_session().await?;
        self.prediction
            .execute_prediction_from_match(&session, &self.registry, command)
            .await
    }

    pub async fn execute_shadow_prediction_from_match(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        let session = self.prediction_session().await?;
        self.prediction
            .execute_shadow_prediction_from_match(&session, &self.registry, command)
            .await
    }

    pub async fn preview_route(
        &self,
        command: RoutePreviewCommand,
    ) -> ApplicationResult<RouteDecision> {
        let session = self.prediction_session().await?;
        self.prediction
            .preview_route(&session, &self.registry, command)
            .await
    }

    pub fn dry_run_default_fixture(&self) -> ApplicationResult<ModelOutput> {
        self.prediction.dry_run_default_fixture(&self.registry)
    }

    pub async fn list_recent_runs(&self, limit: i64) -> ApplicationResult<Vec<ModelRunListItem>> {
        let session = self.prediction_session().await?;
        let items = self.prediction.list_recent_runs(&session, limit).await?;
        items
            .into_iter()
            .map(model_run_list_item_from_port)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub async fn hide_run_from_history(
        &self,
        run_id: Uuid,
        reason: Option<String>,
    ) -> ApplicationResult<()> {
        let session = self.prediction_session().await?;
        self.prediction
            .hide_run_from_history(&session, run_id, reason)
            .await
    }

    pub async fn read_run(&self, run_id: Uuid) -> ApplicationResult<Value> {
        let session = self.prediction_session().await?;
        self.prediction.read_run(&session, run_id).await
    }
}
""",
)
write(
    "crates/application/src/services/prediction/mod.rs",
    "mod facade;\nmod service;\n\npub(crate) use service::PredictionService;",
)

# Extend infrastructure-neutral ports without exposing serde_json::Value.
ports_path = ROOT / "crates/application/src/ports/prediction/mod.rs"
ports = ports_path.read_text(encoding="utf-8")
ports = ports.replace(
    "    P4FreezeTaskTransition, PreparedMatchPredictionInput, RouteDecision,\n",
    "    P4FreezeTaskTransition, PredictionSummary, PreparedMatchPredictionInput, RouteDecision,\n",
)
ports = ports.replace("use uuid::Uuid;\n", "use uuid::Uuid;\n\n#[derive(Debug, Clone)]\npub struct ModelRunHistoryItem {\n    pub id: Uuid,\n    pub match_key: String,\n    pub competition_name: Option<String>,\n    pub home_team_name: Option<String>,\n    pub away_team_name: Option<String>,\n    pub kickoff_time: Option<DateTime<Utc>>,\n    pub snapshot_type: String,\n    pub model_key: String,\n    pub model_version: String,\n    pub parameter_version: String,\n    pub rule_package_name: Option<String>,\n    pub summary: PredictionSummary,\n    pub top_scoreline: Option<String>,\n    pub top_scoreline_probability: Option<f64>,\n    pub created_at: DateTime<Utc>,\n    pub completed_at: Option<DateTime<Utc>>,\n    pub duration_ms: Option<i64>,\n    pub input_readiness_level: String,\n    pub input_readiness_score: Option<i16>,\n    pub input_manifest_sha256: String,\n}\n\n#[derive(Debug, Clone)]\npub struct SerializedModelRun {\n    pub json: String,\n}\n")
ports = ports.replace(
    "    async fn hide_run_from_history(&self, run_id: Uuid, reason: Option<&str>) -> PortResult<()>;\n",
    "    async fn hide_run_from_history(&self, run_id: Uuid, reason: Option<&str>) -> PortResult<()>;\n    async fn list_recent_runs(&self, limit: i64) -> PortResult<Vec<ModelRunHistoryItem>>;\n    async fn read_run_document(&self, run_id: Uuid) -> PortResult<SerializedModelRun>;\n",
)
ports_path.write_text(ports, encoding="utf-8", newline="\n")

# Routing needs the same resolved competition context used by legacy Prediction.
rules_path = ROOT / "crates/application/src/ports/rules/mod.rs"
rules = rules_path.read_text(encoding="utf-8")
rules = rules.replace(
    "    RouteRequest, RulePackageDraft, RulePackageSummary,\n",
    "    ResolvedCompetitionContext, RouteRequest, RulePackageDraft, RulePackageSummary,\n",
)
rules = rules.replace(
    "    async fn resolve_route(&self, request: &RouteRequest) -> PortResult<RouteDecision>;\n",
    "    async fn resolve_competition_context(\n        &self,\n        competition_id: Option<Uuid>,\n        season_id: Option<Uuid>,\n        stage_id: Option<Uuid>,\n        competition_kind: CompetitionKind,\n    ) -> PortResult<ResolvedCompetitionContext>;\n    async fn resolve_route(&self, request: &RouteRequest) -> PortResult<RouteDecision>;\n",
)
rules_path.write_text(rules, encoding="utf-8", newline="\n")

# Implement Prediction ports in the composition adapter.
write(
    "crates/application/src/composition/adapters/prediction.rs",
    """use super::super::port_registry::{
    map_persistence_error, ActiveDatabase, ModelRunListItem,
};
use crate::ports::{
    prediction::{
        ModelRunHistoryItem, ModelRunPort, PredictionInputPort, SerializedModelRun,
    },
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{PredictionSummary, PreparedMatchPredictionInput, RouteDecision};
use football_model_api::{ModelOutput, ModelRequest};
use uuid::Uuid;

#[async_trait]
impl PredictionInputPort for ActiveDatabase {
    async fn prepare_match_input(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
    ) -> PortResult<PreparedMatchPredictionInput> {
        self.transition_store()
            .prepare_match_prediction_input(match_id, snapshot_type, model_family)
            .await
            .map_err(map_persistence_error)
    }

    async fn prepare_match_input_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
        reference_time: DateTime<Utc>,
    ) -> PortResult<PreparedMatchPredictionInput> {
        self.transition_store()
            .prepare_match_prediction_input_at(match_id, snapshot_type, model_family, reference_time)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ModelRunPort for ActiveDatabase {
    async fn save_successful_run(
        &self,
        decision: &RouteDecision,
        request: &ModelRequest,
        output: &ModelOutput,
        duration_ms: i64,
    ) -> PortResult<Uuid> {
        self.transition_store()
            .save_successful_run(decision, request, output, duration_ms)
            .await
            .map_err(map_persistence_error)
    }

    async fn hide_run_from_history(&self, run_id: Uuid, reason: Option<&str>) -> PortResult<()> {
        self.transition_store()
            .hide_run_from_history(run_id, reason)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_recent_runs(&self, limit: i64) -> PortResult<Vec<ModelRunHistoryItem>> {
        self.transition_store()
            .list_recent_runs(limit)
            .await
            .map_err(map_persistence_error)?
            .into_iter()
            .map(|item| {
                let summary: PredictionSummary = serde_json::from_value(item.summary)
                    .map_err(|error| PortError::new(PortErrorKind::Serialization, error.to_string()))?;
                Ok(ModelRunHistoryItem {
                    id: item.id,
                    match_key: item.match_key,
                    competition_name: item.competition_name,
                    home_team_name: item.home_team_name,
                    away_team_name: item.away_team_name,
                    kickoff_time: item.kickoff_time,
                    snapshot_type: item.snapshot_type,
                    model_key: item.model_key,
                    model_version: item.model_version,
                    parameter_version: item.parameter_version,
                    rule_package_name: item.rule_package_name,
                    summary,
                    top_scoreline: item.top_scoreline,
                    top_scoreline_probability: item.top_scoreline_probability,
                    created_at: item.created_at,
                    completed_at: item.completed_at,
                    duration_ms: item.duration_ms,
                    input_readiness_level: item.input_readiness_level,
                    input_readiness_score: item.input_readiness_score,
                    input_manifest_sha256: item.input_manifest_sha256,
                })
            })
            .collect()
    }

    async fn read_run_document(&self, run_id: Uuid) -> PortResult<SerializedModelRun> {
        let value = self
            .transition_store()
            .read_run(run_id)
            .await
            .map_err(map_persistence_error)?;
        let json = serde_json::to_string(&value)
            .map_err(|error| PortError::new(PortErrorKind::Serialization, error.to_string()))?;
        Ok(SerializedModelRun { json })
    }
}

pub(crate) fn model_run_list_item_from_port(
    item: ModelRunHistoryItem,
) -> Result<ModelRunListItem, serde_json::Error> {
    Ok(ModelRunListItem {
        id: item.id,
        match_key: item.match_key,
        competition_name: item.competition_name,
        home_team_name: item.home_team_name,
        away_team_name: item.away_team_name,
        kickoff_time: item.kickoff_time,
        snapshot_type: item.snapshot_type,
        model_key: item.model_key,
        model_version: item.model_version,
        parameter_version: item.parameter_version,
        rule_package_name: item.rule_package_name,
        summary: serde_json::to_value(item.summary)?,
        top_scoreline: item.top_scoreline,
        top_scoreline_probability: item.top_scoreline_probability,
        created_at: item.created_at,
        completed_at: item.completed_at,
        duration_ms: item.duration_ms,
        input_readiness_level: item.input_readiness_level,
        input_readiness_score: item.input_readiness_score,
        input_manifest_sha256: item.input_manifest_sha256,
    })
}
""",
)

# Wire the new adapter and rule context method.
adapters_mod = ROOT / "crates/application/src/composition/adapters/mod.rs"
adapters = adapters_mod.read_text(encoding="utf-8")
if "mod prediction;" not in adapters:
    adapters = adapters.replace("mod players;", "mod players;\nmod prediction;")
adapters_mod.write_text(adapters, encoding="utf-8", newline="\n")

port_registry_path = ROOT / "crates/application/src/composition/port_registry.rs"
port_registry = port_registry_path.read_text(encoding="utf-8")
port_registry = port_registry.replace(
    "    CompetitionRecord, RoundDraft, RoundRecord, RouteDecision, RouteRequest, RulePackageDraft,\n",
    "    CompetitionRecord, ResolvedCompetitionContext, RoundDraft, RoundRecord, RouteDecision, RouteRequest, RulePackageDraft,\n",
)
needle = "    async fn resolve_route(&self, request: &RouteRequest) -> PortResult<RouteDecision> {"
if needle not in port_registry:
    raise RuntimeError("RuleRoutingPort resolve_route impl marker not found")
context_impl = """    async fn resolve_competition_context(
        &self,
        competition_id: Option<Uuid>,
        season_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        competition_kind: CompetitionKind,
    ) -> PortResult<ResolvedCompetitionContext> {
        self.store
            .resolve_competition_context(competition_id, season_id, stage_id, competition_kind)
            .await
            .map_err(map_persistence_error)
    }

"""
port_registry = port_registry.replace(needle, context_impl + needle, 1)
port_registry_path.write_text(port_registry, encoding="utf-8", newline="\n")

# Service/composition ownership.
services_mod = ROOT / "crates/application/src/services/mod.rs"
services = services_mod.read_text(encoding="utf-8")
if "pub(crate) mod prediction;" not in services:
    services = services.replace("pub(crate) mod players;", "pub(crate) mod players;\npub(crate) mod prediction;")
services_mod.write_text(services, encoding="utf-8", newline="\n")

use_cases_mod = ROOT / "crates/application/src/use_cases/mod.rs"
use_cases = use_cases_mod.read_text(encoding="utf-8")
if "pub(crate) mod prediction;" not in use_cases:
    use_cases = use_cases.replace("pub(crate) mod players;", "pub(crate) mod players;\npub(crate) mod prediction;")
use_cases_mod.write_text(use_cases, encoding="utf-8", newline="\n")

service_path = ROOT / "crates/application/src/service/application_service.rs"
service_text = service_path.read_text(encoding="utf-8")
service_text = service_text.replace(
    "    players::PlayerService, rules::RulesService, teams::TeamService,\n",
    "    players::PlayerService, prediction::PredictionService, rules::RulesService, teams::TeamService,\n",
)
service_text = service_text.replace(
    "    pub(crate) players: PlayerService,\n    pub(crate) lineups: LineupService,\n",
    "    pub(crate) players: PlayerService,\n    pub(crate) lineups: LineupService,\n    pub(crate) prediction: PredictionService,\n",
)
service_text = service_text.replace(
    "let (registry, database, competition, rules, teams, players, lineups, p4_worker_running) =\n            ApplicationComposition::new().into_parts();",
    "let (registry, database, competition, rules, teams, players, lineups, prediction, p4_worker_running) =\n            ApplicationComposition::new().into_parts();",
)
service_text = service_text.replace(
    "            lineups,\n            p4_worker_running,\n",
    "            lineups,\n            prediction,\n            p4_worker_running,\n",
)
service_path.write_text(service_text, encoding="utf-8", newline="\n")

composition_path = ROOT / "crates/application/src/composition/application_composition.rs"
composition = composition_path.read_text(encoding="utf-8")
composition = composition.replace(
    "    players::PlayerService, rules::RulesService, teams::TeamService,\n",
    "    players::PlayerService, prediction::PredictionService, rules::RulesService, teams::TeamService,\n",
)
composition = composition.replace(
    "    players: PlayerService,\n    lineups: LineupService,\n",
    "    players: PlayerService,\n    lineups: LineupService,\n    prediction: PredictionService,\n",
)
composition = composition.replace(
    "            lineups: LineupService::new(),\n            p4_worker_running: AtomicBool::new(false),\n",
    "            lineups: LineupService::new(),\n            prediction: PredictionService::new(),\n            p4_worker_running: AtomicBool::new(false),\n",
)
composition = composition.replace(
    "        LineupService,\n        AtomicBool,\n",
    "        LineupService,\n        PredictionService,\n        AtomicBool,\n",
)
composition = composition.replace(
    "            self.lineups,\n            self.p4_worker_running,\n",
    "            self.lineups,\n            self.prediction,\n            self.p4_worker_running,\n",
)
composition_path.write_text(composition, encoding="utf-8", newline="\n")

# Remove only the legacy Prediction owner; P4 orchestration/workbench remain for phase 2.
lib_path = ROOT / "crates/application/src/lib.rs"
lib = lib_path.read_text(encoding="utf-8")
lib = lib.replace("mod prediction;\n", "")
lib = lib.replace(
    "prediction::ensure_match_input_id",
    "use_cases::prediction::shared::routing::ensure_match_input_id",
)
lib_path.write_text(lib, encoding="utf-8", newline="\n")
SRC.unlink()

print("R3-06 phase 1 migration generated")
