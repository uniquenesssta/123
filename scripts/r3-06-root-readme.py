from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
README = ROOT / "README.md"
text = README.read_text(encoding="utf-8")

old_summary = "R3-05 状态为 `DONE`，R3-06 Prediction Service 已开放为 `READY`。"
new_summary = (
    "R3-05 状态为 `DONE`。R3-06 Prediction Service 已进入 `IN_PROGRESS`："
    "Atomic Task 1 已完成 Prediction Core 模块化迁移；Atomic Task 2A 已完成 P4 planning / freeze readiness / read-only workspace 迁移及专项硬门禁。"
    "Research 冲突写入、Evidence/Fact 写入与联网 Research 执行仍明确保留给 R3-07，R3-06 尚未关闭。"
)
if text.count(old_summary) != 1:
    raise RuntimeError("root README R3 summary marker mismatch")
text = text.replace(old_summary, new_summary, 1)

old_index = "R3-06 Prediction Service 为 `READY`。详细状态见 `docs/modular-rewrite/R03-application-services/README.md`。"
new_index = "R3-06 Prediction Service 为 `IN_PROGRESS`。详细状态见 `docs/modular-rewrite/R03-application-services/README.md`。"
if text.count(old_index) != 1:
    raise RuntimeError("root README R3 index marker mismatch")
text = text.replace(old_index, new_index, 1)

marker = "\n## R2-04 Lineup 与 Match\n"
if text.count(marker) != 1:
    raise RuntimeError("root README insertion marker mismatch")
section = r'''

## R3-06 Prediction Service（IN_PROGRESS）

- R3-06 在独立分支 `rewrite/r3-06-prediction-service` 上实施，未修改 `new-C` 的 R3-05 已验收基线。Atomic Task 1 已将 Prediction Core 的推演执行、readiness、route preview、formal/shadow stored-match execution、dry-run 与运行历史职责迁入 `services/prediction/`、`use_cases/prediction/`，并通过既有 Ports 保持 ApplicationService / Tauri 公共调用语义；模型执行继续只经 `football-model-api` 边界，不修改或复制模型实现。
- Atomic Task 2A 仅迁移 Prediction 所属的 P4 horizon planning、freeze task list/read/events、freeze readiness、match/task workspace 只读职责；`resolve_p4_conflict`、联网 Research 执行、Evidence/Fact 写入和 Research artifact 写入仍保留给 R3-07，不因旧文件混合职责而提前迁移。
- 2A 专项 Windows run `31266144950` / job `93124468057` 已通过 Application Ports、完整 architecture、rustfmt、`cargo check --locked -p football-application` 与 `cargo test --locked -p football-application`，Application tests 33/33 通过。
- 模块化删除旧 `crates/application/src/prediction.rs` 后，确认并修复 3 个历史验证器的旧 owner 路径：默认战术角色、比赛工作流、历史比分验证器均改读当前 Prediction Service / Use Case 权威模块；原业务断言未删除或放宽，其中比赛工作流与历史验证改为递归扫描完整 Prediction 模块树。
- 2A 编译期确认的未使用 import 已直接清理，不增加 lint 抑制。warning-cleanup Windows run `31266871976` / job `93126329974` 已通过 Application Ports、architecture、rustfmt、`cargo clippy --locked -p football-application --all-targets -- -D warnings` 与 Application tests；测试专用 `P4Horizon` / `is_p4_model` 仅移入 `#[cfg(test)]` 作用域。
- clean 源码头 `7e3f43d805b22fceffc6a367392ad9fa1eabef36` 已删除 2A 与 warning-cleanup 的临时 workflow / Python 脚本。完整 Public Platform CI 仍需在最终状态提交上通过后才能关闭 2A；当前不得将 R3-06 或 R3-07 标记为 DONE / READY。
'''
text = text.replace(marker, section + marker, 1)
README.write_text(text, encoding="utf-8", newline="\n")
print("root README R3-06 status updated")
