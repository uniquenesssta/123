from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

ROOT_OLD = "- Atomic Task 2B 已完成专项实施并通过 Windows 硬门禁 run `31289363055` / job `93183820380`：P4 正式冻结状态机、固定路由复核、29+2=31 字段快照投影、模型概率矩阵投影与不可变正式快照写入已迁入 Prediction Service / Use Case，并新增独立 `P4FreezeExecutionPort`；旧混合 P4 worker 仅在 freeze job 分支委托 Prediction Service。专项验证已通过 `verify-r3-06-p4-freeze`、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。OpenAI Research、Evidence/Fact 写入、冲突人工覆盖仍保留在 R3-07 边界。当前只等待 clean 源码树上的最终 Public Platform CI，因此 R3-06 继续为 `IN_PROGRESS`、R3-07 继续为 `BLOCKED`。"
ROOT_NEW = "- Atomic Task 2B 已正式关闭为 `DONE`。实施提交 `0d691114e67116fb9f03e4cd0fb04c6a819d4254` 已将 P4 正式冻结状态机、固定路由复核、29+2=31 字段快照投影、模型概率矩阵投影与不可变正式快照写入迁入 Prediction Service / Use Case，并新增独立 `P4FreezeExecutionPort`；旧混合 P4 worker 仅在 freeze job 分支委托 Prediction Service，OpenAI Research、Evidence/Fact 写入与冲突人工覆盖继续保留在 R3-07 边界。专项 Windows hard gate run `31289363055` / job `93183820380` 已通过 `verify-r3-06-p4-freeze`、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。clean 源码树验收提交 `b0fe2fe5cf3a24e7a8894b8f44061570afa3e35d` 的 Public Platform CI run `31289854065` 已全部通过：architecture job `93185076242` 与 Windows Automated job `93185076247` 均为 SUCCESS，完整 frontend、17 个截图回归视口、TypeScript、Vite production build、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Tauri Windows release 构建及 release runtime 日志验收均为 PASS；Application tests 33/33 通过，runtime 自动验收为 7 条日志 / 3 个完成操作。artifact `9031315604` 大小 `14249898` 字节，SHA-256 `438a352f8d81cee9b044d9c1d9a36682f1df435fa513eb7768bbd875322e89bb`。18 个真实 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 按既有安全设计保持 `ignored`，未记为已执行。2B 临时 workflow / Python 脚本均已清理；R3-06 继续为 `IN_PROGRESS`，R3-07 继续为 `BLOCKED`。"

R3_OLD = "- Atomic Task 2B 已完成专项实施并通过 Windows hard gate run `31289363055` / job `93183820380`：P4 freeze execution 由 `services/prediction` / `use_cases/prediction/execute_p4_freeze/` 接管，通过独立 `P4FreezeExecutionPort` 访问路由事实、已有冻结快照与不可变快照写入；专项 `verify-r3-06-p4-freeze`、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests 均通过。旧 `p4_orchestration.rs` 的 Research worker 与 OpenAI Research / Evidence / Fact / conflict mutation 继续留给 R3-07。当前仅等待 clean 源码树上的最终 Public Platform CI，R3-06 保持 `IN_PROGRESS`、R3-07 保持 `BLOCKED`。"
R3_NEW = "- Atomic Task 2B 已正式关闭为 `DONE`。实施提交 `0d691114e67116fb9f03e4cd0fb04c6a819d4254` 将 P4 freeze execution 迁入 `services/prediction` / `use_cases/prediction/execute_p4_freeze/`，通过独立 `P4FreezeExecutionPort` 访问路由事实、已有冻结快照与不可变快照写入；旧 `p4_orchestration.rs` 的 Research worker 与 OpenAI Research / Evidence / Fact / conflict mutation 继续留给 R3-07。专项 run `31289363055` / job `93183820380` 已通过 freeze verifier、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。最终 clean-tree Public Platform CI run `31289854065` 已通过 architecture job `93185076242` 和 Windows Automated job `93185076247`，覆盖完整 frontend、17 个截图回归视口、TypeScript、Vite、Rust fmt、workspace Clippy/tests、Tauri Windows release 与 release runtime smoke；Application tests 33/33，runtime 为 7 条日志 / 3 个完成操作。artifact `9031315604` 大小 `14249898` 字节，SHA-256 `438a352f8d81cee9b044d9c1d9a36682f1df435fa513eb7768bbd875322e89bb`。18 个 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 保持 `ignored`。2B 临时 workflow / Python 脚本均已清理；R3-06 保持 `IN_PROGRESS`，R3-07 保持 `BLOCKED`，可继续 R3-06 下一 Atomic Task。"

for relative, old, new in [
    ("README.md", ROOT_OLD, ROOT_NEW),
    ("docs/modular-rewrite/R03-application-services/README.md", R3_OLD, R3_NEW),
]:
    path = ROOT / relative
    text = path.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise RuntimeError(f"2B close marker mismatch in {relative}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")

print("R3-06 Atomic Task 2B final status recorded")
