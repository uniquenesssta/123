from pathlib import Path

GENERATOR = Path(__file__).resolve().with_name("r3-07-at4.py")
text = GENERATOR.read_text(encoding="utf-8")

old = '''replace_once(
    "crates/application/src/p4_workbench.rs",
    "reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;",
    "reconcile_p4_task_after_manual_decision(self, &store, task.id, research_run_id).await?;",
    "manual conflict idempotent reconciliation call",
)
replace_once(
    "crates/application/src/p4_workbench.rs",
    "reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;",
    "reconcile_p4_task_after_manual_decision(self, &store, task.id, research_run_id).await?;",
    "manual conflict post-write reconciliation call",
)
'''
new = '''workbench_path = ROOT / "crates/application/src/p4_workbench.rs"
workbench = workbench_path.read_text(encoding="utf-8")
old_reconcile_call = "reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;"
new_reconcile_call = "reconcile_p4_task_after_manual_decision(self, &store, task.id, research_run_id).await?;"
if workbench.count(old_reconcile_call) != 2:
    raise RuntimeError("AT4 anchor mismatch: expected exactly two manual conflict reconciliation calls")
workbench_path.write_text(
    workbench.replace(old_reconcile_call, new_reconcile_call, 2),
    encoding="utf-8",
    newline="\\n",
)
'''
if text.count(old) != 1:
    raise RuntimeError("AT4 fix anchor mismatch")
GENERATOR.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")
print("AT4 generator patched for the two intentional workbench reconciliation call sites")
