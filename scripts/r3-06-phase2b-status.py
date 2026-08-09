from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

ROOT_OLD = "- Atomic Task 2B 已进入实施：P4 正式冻结状态机、固定路由复核、29+2=31 字段快照投影、模型概率矩阵投影与不可变正式快照写入已迁入 Prediction Service / Use Case，并新增独立 `P4FreezeExecutionPort`；旧混合 P4 worker 仅在 freeze job 分支委托 Prediction Service。OpenAI Research、Evidence/Fact 写入、冲突人工覆盖仍保留在 R3-07 边界。本节点尚待 2B 硬门禁与最终 Public Platform CI，因此 R3-06 继续为 `IN_PROGRESS`。"
ROOT_NEW = "- Atomic Task 2B 已完成专项实施并通过 Windows 硬门禁 run `31289363055` / job `93183820380`：P4 正式冻结状态机、固定路由复核、29+2=31 字段快照投影、模型概率矩阵投影与不可变正式快照写入已迁入 Prediction Service / Use Case，并新增独立 `P4FreezeExecutionPort`；旧混合 P4 worker 仅在 freeze job 分支委托 Prediction Service。专项验证已通过 `verify-r3-06-p4-freeze`、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。OpenAI Research、Evidence/Fact 写入、冲突人工覆盖仍保留在 R3-07 边界。当前只等待 clean 源码树上的最终 Public Platform CI，因此 R3-06 继续为 `IN_PROGRESS`、R3-07 继续为 `BLOCKED`。"

R3_OLD = "- Atomic Task 2B 已进入实施：P4 freeze execution 由 `services/prediction` / `use_cases/prediction/execute_p4_freeze/` 接管，通过独立 `P4FreezeExecutionPort` 访问路由事实、已有冻结快照与不可变快照写入；旧 `p4_orchestration.rs` 的 Research worker 与 OpenAI Research / Evidence / Fact / conflict mutation 继续留给 R3-07。当前等待 2B 硬门禁与最终 Public Platform CI，R3-06 保持 `IN_PROGRESS`、R3-07 保持 `BLOCKED`。"
R3_NEW = "- Atomic Task 2B 已完成专项实施并通过 Windows hard gate run `31289363055` / job `93183820380`：P4 freeze execution 由 `services/prediction` / `use_cases/prediction/execute_p4_freeze/` 接管，通过独立 `P4FreezeExecutionPort` 访问路由事实、已有冻结快照与不可变快照写入；专项 `verify-r3-06-p4-freeze`、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests 均通过。旧 `p4_orchestration.rs` 的 Research worker 与 OpenAI Research / Evidence / Fact / conflict mutation 继续留给 R3-07。当前仅等待 clean 源码树上的最终 Public Platform CI，R3-06 保持 `IN_PROGRESS`、R3-07 保持 `BLOCKED`。"

for relative, old, new in [
    ("README.md", ROOT_OLD, ROOT_NEW),
    ("docs/modular-rewrite/R03-application-services/README.md", R3_OLD, R3_NEW),
]:
    path = ROOT / relative
    text = path.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise RuntimeError(f"status marker mismatch in {relative}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

print("R3-06 Atomic Task 2B targeted validation status recorded")
