from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

root_readme = ROOT / "README.md"
text = root_readme.read_text(encoding="utf-8")
if "Atomic Task 2B 已进入实施" not in text:
    marker = "\n\n## R2-04 Lineup 与 Match\n"
    if text.count(marker) != 1:
        raise RuntimeError("root README R3-06 insertion marker mismatch")
    entry = (
        "\n- Atomic Task 2B 已进入实施：P4 正式冻结状态机、固定路由复核、29+2=31 字段快照投影、模型概率矩阵投影与不可变正式快照写入已迁入 Prediction Service / Use Case，并新增独立 `P4FreezeExecutionPort`；旧混合 P4 worker 仅在 freeze job 分支委托 Prediction Service。OpenAI Research、Evidence/Fact 写入、冲突人工覆盖仍保留在 R3-07 边界。本节点尚待 2B 硬门禁与最终 Public Platform CI，因此 R3-06 继续为 `IN_PROGRESS`。"
    )
    text = text.replace(marker, entry + marker, 1)
root_readme.write_text(text, encoding="utf-8", newline="\n")

r3_readme = ROOT / "docs/modular-rewrite/R03-application-services/README.md"
text = r3_readme.read_text(encoding="utf-8").rstrip()
if "Atomic Task 2B 已进入实施" not in text:
    text += (
        "\n- Atomic Task 2B 已进入实施：P4 freeze execution 由 `services/prediction` / `use_cases/prediction/execute_p4_freeze/` 接管，通过独立 `P4FreezeExecutionPort` 访问路由事实、已有冻结快照与不可变快照写入；旧 `p4_orchestration.rs` 的 Research worker 与 OpenAI Research / Evidence / Fact / conflict mutation 继续留给 R3-07。当前等待 2B 硬门禁与最终 Public Platform CI，R3-06 保持 `IN_PROGRESS`、R3-07 保持 `BLOCKED`。\n"
    )
r3_readme.write_text(text, encoding="utf-8", newline="\n")

print("R3-06 Atomic Task 2B documentation status recorded")
