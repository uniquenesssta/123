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

GENERATOR.write_text(text, encoding="utf-8", newline="\n")
