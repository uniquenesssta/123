from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: Path, old: str, new: str, label: str) -> None:
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


root_readme = ROOT / "README.md"

replace_once(
    root_readme,
    "R3-06 Prediction Service 已进入 `IN_PROGRESS`：Atomic Task 1 已完成 Prediction Core 模块化迁移；Atomic Task 2A 已完成 P4 planning / freeze readiness / read-only workspace 迁移及专项硬门禁。Research 冲突写入、Evidence/Fact 写入与联网 Research 执行仍明确保留给 R3-07，R3-06 尚未关闭。",
    "R3-06 Prediction Service 已完成并关闭为 `DONE`；R3-07 Research Service 已完成并关闭为 `DONE`。R3-08 Review / Postmatch / Analytics Services 仍按阶段索引保持 `BLOCKED`。",
    "root README R3 summary",
)

replace_once(
    root_readme,
    "Atomic Task 5 已进入 `IN_PROGRESS`：人工冲突裁决迁入 ResearchService / `use_cases/research/p4_manual_conflict/`，只经既有 Prediction/Job/Research Ports 协作；SQL、Schema、迁移、生产依赖、公共 `resolve_p4_conflict` 契约、截止时间、幂等、append-only 与状态机语义保持不变。Windows hard gate run `31318631427` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。AT5 clean implementation 将在本次 hard gate 后由同一 workflow 提交；正式 Public Platform Windows Automated 通过前 R3-07 保持 `IN_PROGRESS`。",
    "Atomic Task 5 已正式关闭为 `DONE`：人工冲突裁决已迁入 ResearchService / `use_cases/research/p4_manual_conflict/`，只经既有 Prediction/Job/Research Ports 协作；SQL、Schema、迁移、生产依赖、公共 `resolve_p4_conflict` 契约、截止时间、幂等、append-only 与状态机语义保持不变。Windows hard gate run `31318631427` / job `93257903560` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean implementation `c1549738041c0e6d3cb046c16fa193c47a14553f`。该 clean HEAD 的 Public Platform CI run `31319176935` / Windows Automated job `93259283555` 已整体 `SUCCESS`，validation evidence upload 成功；artifact `9039991499` 大小 `14210568` 字节，SHA-256 `e48cf0649449fc74488a60ea560a43a224c3f49fe3f9723ad1e039cc06f79d75`。AT5 临时 workflow / generator / fix / marker 已清理；R3-07 五个 Atomic Tasks 全部完成，Research Service 状态正式关闭为 `DONE`。",
    "root README AT5 closeout",
)

r03_readme = ROOT / "docs/modular-rewrite/R03-application-services/README.md"
replace_once(
    r03_readme,
    "| R3-07 | Research Service | IN_PROGRESS |",
    "| R3-07 | Research Service | DONE |",
    "R03 task table R3-07 status",
)

replace_once(
    r03_readme,
    "Atomic Task 5 已进入 `IN_PROGRESS`：人工冲突裁决迁入 ResearchService / `use_cases/research/p4_manual_conflict/`，公共 `resolve_p4_conflict` 与全部既有人工裁决语义保持不变。Windows hard gate run `31318631427` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。AT5 clean implementation 将在本次 hard gate 后由同一 workflow 提交；正式 Public Platform Windows Automated 通过前 R3-07 保持 `IN_PROGRESS`。",
    "Atomic Task 5 已正式关闭为 `DONE`：人工冲突裁决已迁入 ResearchService / `use_cases/research/p4_manual_conflict/`，公共 `resolve_p4_conflict` 与全部既有人工裁决语义保持不变。Windows hard gate run `31318631427` / job `93257903560` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean implementation `c1549738041c0e6d3cb046c16fa193c47a14553f`。该 clean HEAD 的 Public Platform CI run `31319176935` / Windows Automated job `93259283555` 已整体 `SUCCESS`，validation evidence upload 成功；artifact `9039991499` 大小 `14210568` 字节，SHA-256 `e48cf0649449fc74488a60ea560a43a224c3f49fe3f9723ad1e039cc06f79d75`。AT5 临时 workflow / generator / fix / marker 已清理；R3-07 五个 Atomic Tasks 全部完成并正式关闭为 `DONE`，R3-08 仍按阶段索引保持 `BLOCKED`。",
    "R03 AT5 closeout",
)

print("R3-07 closeout records updated")
