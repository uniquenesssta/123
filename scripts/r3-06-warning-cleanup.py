from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise RuntimeError(f"expected exactly one match in {path}: {old!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


replace_once(
    "crates/application/src/p4_orchestration.rs",
    "use crate::built_in_artifacts::{\n    P4_RESEARCH_SCHEMA_ARTIFACT_VERSION as RESEARCH_SCHEMA_VERSION,\n    P4_RESEARCH_SCHEMA_KEY as RESEARCH_SCHEMA_KEY,\n    P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION as SNAPSHOT_SCHEMA_VERSION,\n    P4_SNAPSHOT_SCHEMA_KEY as SNAPSHOT_SCHEMA_KEY,\n};\nuse crate::model_shell::P4_MODEL_ID;\nuse crate::use_cases::prediction::shared::p4_planning::{\n    canonical_fact_keys, horizon_priority, is_p4_model,\n};\nuse crate::PersistenceStore;\nuse chrono::{Duration, Utc};\nuse football_domain::{\n    EnqueueJobDraft, EvidenceVerificationState, P4FreezeReadiness, P4FreezeTaskDraft,\n    P4FreezeTaskEventRecord, P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition,\n    P4Horizon, P4RoutedFact, PlanP4HorizonsCommand, PrematchSnapshotDraft, ResearchRunDraft,\n    ResearchRunStatus, SnapshotFeatureDraft, SnapshotProbabilityDraft, SnapshotSourceKind,\n    P4_FREEZE_GRACE_MINUTES, P4_ORCHESTRATION_PLANNER_VERSION, P4_RESEARCH_LEAD_MINUTES,\n};",
    "use crate::built_in_artifacts::{\n    P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION as SNAPSHOT_SCHEMA_VERSION,\n    P4_SNAPSHOT_SCHEMA_KEY as SNAPSHOT_SCHEMA_KEY,\n};\nuse crate::use_cases::prediction::shared::p4_planning::{canonical_fact_keys, horizon_priority};\nuse crate::PersistenceStore;\nuse chrono::Utc;\nuse football_domain::{\n    EnqueueJobDraft, EvidenceVerificationState, P4FreezeReadiness, P4FreezeTaskRecord,\n    P4FreezeTaskState, P4FreezeTaskTransition, P4RoutedFact, PrematchSnapshotDraft,\n    ResearchRunDraft, ResearchRunStatus, SnapshotFeatureDraft, SnapshotProbabilityDraft,\n    SnapshotSourceKind, P4_ORCHESTRATION_PLANNER_VERSION,\n};",
)
replace_once(
    "crates/application/src/p4_orchestration.rs",
    "mod tests {\n    use super::*;",
    "mod tests {\n    use super::*;\n    use crate::use_cases::prediction::shared::p4_planning::is_p4_model;\n    use football_domain::P4Horizon;",
)
replace_once(
    "crates/application/src/p4_workbench.rs",
    "    P4ManualRouteOverrideDraft, P4MatchWorkspace, P4TaskWorkspace, ResearchRunEventDraft,",
    "    P4ManualRouteOverrideDraft, P4TaskWorkspace, ResearchRunEventDraft,",
)
replace_once(
    "crates/application/src/use_cases/prediction/plan_p4_horizons/mod.rs",
    "    P4FreezeTaskTransition, P4Horizon, PlanP4HorizonsCommand, RouteRequest,",
    "    P4FreezeTaskTransition, P4Horizon, PlanP4HorizonsCommand,",
)

print("R3-06 P4 migration warning cleanup applied")
