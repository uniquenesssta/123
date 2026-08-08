from pathlib import Path

GENERATOR = Path(__file__).resolve().with_name("r3-06-phase2a.py")
text = GENERATOR.read_text(encoding="utf-8")

old_plan = "f'''use super::super::P4PlanningAccess;\nuse super::super::shared::p4_planning::{{"
new_plan = "f'''use super::P4PlanningAccess;\nuse super::shared::p4_planning::{{"
if old_plan not in text:
    raise RuntimeError("phase2a planner import template not found")
text = text.replace(old_plan, new_plan, 1)

old_target = '    "        ModelRunHistoryItem, ModelRunPort, PredictionInputPort, SerializedModelRun,\\n",'
new_target = '    "    prediction::{ModelRunHistoryItem, ModelRunPort, PredictionInputPort, SerializedModelRun},\\n",'
old_replacement = '    "        ModelRunHistoryItem, ModelRunPort, PredictionInputPort, PredictionWorkflowPort,\\n        SerializedModelRun,\\n",'
new_replacement = '    "    prediction::{\\n        ModelRunHistoryItem, ModelRunPort, PredictionInputPort, PredictionWorkflowPort,\\n        SerializedModelRun,\\n    },\\n",'
if old_target not in text or old_replacement not in text:
    raise RuntimeError("phase2a prediction adapter import template not found")
text = text.replace(old_target, new_target, 1)
text = text.replace(old_replacement, new_replacement, 1)

planner_rewrites = {
    'plan = plan.replace("store.p4_planning_match_context(", "store.planning_match_context(")':
        'plan = plan.replace(".p4_planning_match_context(", ".planning_match_context(")',
    'plan = plan.replace("store.read_schema_version_by_key(", "store.read_schema(")':
        'plan = plan.replace(".read_schema_version_by_key(", ".read_schema(")',
    'plan = plan.replace("store.find_p4_freeze_task_by_idempotency(", "store.find_freeze_task_by_idempotency(")':
        'plan = plan.replace(".find_p4_freeze_task_by_idempotency(", ".find_freeze_task_by_idempotency(")',
    'plan = plan.replace("store.create_p4_freeze_task(", "store.create_freeze_task(")':
        'plan = plan.replace(".create_p4_freeze_task(", ".create_freeze_task(")',
    'plan = plan.replace("store.enqueue_job(", "store.enqueue(")':
        'plan = plan.replace(".enqueue_job(", ".enqueue(")',
    'plan = plan.replace("store.transition_p4_freeze_task(&P4FreezeTaskTransition {", "store.transition_freeze_task(task.id, &P4FreezeTaskTransition {")':
        'plan = plan.replace(".transition_p4_freeze_task(&P4FreezeTaskTransition {", ".transition_freeze_task(task.id, &P4FreezeTaskTransition {")',
}
for old, new in planner_rewrites.items():
    if old not in text:
        raise RuntimeError(f"phase2a planner rewrite template not found: {old}")
    text = text.replace(old, new, 1)

GENERATOR.write_text(text, encoding="utf-8", newline="\n")
