from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]
BASE = ROOT / "crates/application/src/use_cases/prediction"


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def write(rel: str, text: str) -> None:
    (ROOT / rel).write_text(text, encoding="utf-8", newline="\n")


def patch(rel: str, fn) -> None:
    text = read(rel)
    updated = fn(text)
    if updated == text:
        raise RuntimeError(f"no change applied to {rel}")
    write(rel, updated)


# Direct children of use_cases::prediction only need one super hop.
for rel in [
    "crates/application/src/use_cases/prediction/execute_prediction/mod.rs",
    "crates/application/src/use_cases/prediction/execute_prediction_from_match/mod.rs",
    "crates/application/src/use_cases/prediction/inspect_match_prediction_readiness/mod.rs",
    "crates/application/src/use_cases/prediction/preview_route/mod.rs",
]:
    path = ROOT / rel
    text = path.read_text(encoding="utf-8")
    text = text.replace("super::super::", "super::")
    text = re.sub(r"self\s*\.\s*registry", "registry", text)
    path.write_text(text, encoding="utf-8", newline="\n")

# Preserve the legacy formal/shadow contract exactly: formal persists, shadow does not.
rel = "crates/application/src/use_cases/prediction/execute_prediction_from_match/mod.rs"
text = read(rel)
text = text.replace(
    "execute_with_mode(port, registry, command, false).await",
    "execute_with_mode(port, registry, command, true).await",
    1,
)
text = text.replace(
    "execute_with_mode(port, registry, command, true).await",
    "execute_with_mode(port, registry, command, false).await",
    1,
)
# The two sequential replacements above can touch the same text; force wrapper values by function block.
text = re.sub(
    r"(pub\(crate\) async fn execute_formal[\s\S]*?execute_with_mode\(port, registry, command, )(?:true|false)(\)\.await\n\})",
    r"\g<1>true\2",
    text,
    count=1,
)
text = re.sub(
    r"(pub\(crate\) async fn execute_shadow[\s\S]*?execute_with_mode\(port, registry, command, )(?:true|false)(\)\.await\n\})",
    r"\g<1>false\2",
    text,
    count=1,
)
text = text.replace("    shadow: bool,", "    persist_run: bool,")
text = re.sub(
    r"self\s*\.\s*inspect_match_prediction_readiness\(command\.clone\(\)\)\s*\.await\?",
    "super::inspect_match_prediction_readiness::execute(port, registry, command.clone()).await?",
    text,
)
text = re.sub(
    r"self\s*\.\s*execute_prediction_internal\(",
    "super::execute_prediction::execute_internal(port, registry, ",
    text,
)
text = text.replace("let store = self.active_store().await?;", "let store = port;")
if "self." in text:
    raise RuntimeError("stored-match use case still contains ApplicationService self access")
write(rel, text)

# Readiness needs the complete legacy domain/helper context.
rel = "crates/application/src/use_cases/prediction/inspect_match_prediction_readiness/mod.rs"
text = read(rel)
text = text.replace(
    "ensure_model_selection_registered, normalize_model_selection, route_identity_manifest,",
    "ensure_model_selection_registered, normalize_model_selection, route_identity_manifest,\n    validate_snapshot_type,",
)
text = text.replace(
    "MatchPredictionReadiness, PredictionReadinessCheck, PredictionReadinessCheckStatus,",
    "CompetitionKind, MatchContext, MatchPredictionReadiness, PredictionReadinessCheck,\n    PredictionReadinessCheckStatus,",
)
if "self." in text:
    raise RuntimeError("readiness use case still contains ApplicationService self access")
write(rel, text)

# Route preview uses kickoff parsing and preserves validation error semantics.
rel = "crates/application/src/use_cases/prediction/preview_route/mod.rs"
text = read(rel)
text = text.replace(
    "ensure_model_selection_registered, match_context_from_command, normalize_model_selection,",
    "ensure_model_selection_registered, normalize_model_selection, parse_kickoff,",
)
text = text.replace(
    "use crate::{ApplicationResult, RoutePreviewCommand};",
    "use crate::{ApplicationError, ApplicationResult, RoutePreviewCommand};",
)
if "self." in text:
    raise RuntimeError("route preview use case still contains ApplicationService self access")
write(rel, text)

# Dry-run is registry-only but still needs the legacy context builders.
rel = "crates/application/src/use_cases/prediction/dry_run_default_fixture/mod.rs"
text = read(rel)
text = re.sub(r"self\s*\.\s*registry", "registry", text)
text = text.replace(
    "use crate::model_shell::P4_MODEL_ID;",
    "use crate::model_shell::P4_MODEL_ID;\nuse super::shared::routing::{nested_required_string, parse_kickoff, required_string};",
)
text = text.replace(
    "use football_domain::{ModelIdentity};",
    "use football_domain::{CompetitionKind, MatchContext, ModelIdentity};",
)
text = text.replace(
    "use football_model_api::{ModelOutput, ModelRequest};",
    "use football_model_api::{ModelOutput, ModelRequest};\nuse serde_json::Value;",
)
if "self." in text:
    raise RuntimeError("dry-run use case still contains ApplicationService self access")
write(rel, text)

# Core execution must be fully detached from ApplicationService.
rel = "crates/application/src/use_cases/prediction/execute_prediction/mod.rs"
text = read(rel)
text = re.sub(r"self\s*\.\s*registry", "registry", text)
text = text.replace("let store = self.active_store().await?;", "let store = port;")
if "self." in text:
    raise RuntimeError("core prediction use case still contains ApplicationService self access")
write(rel, text)

# Export the adapter conversion at the composition boundary, not from the facade implementation.
rel = "crates/application/src/composition/mod.rs"
text = read(rel)
export = "pub(crate) use adapters::prediction::model_run_list_item_from_port;"
if export not in text:
    text = text.rstrip() + "\n\n" + export + "\n"
write(rel, text)

print("R3-06 phase 1 deterministic fixes applied")
