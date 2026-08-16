from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def replace_once(path: str, old: str, new: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"expected exactly one match in {path}, found {count}: {old[:100]!r}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8")


# Root README: record only facts that have now actually completed.
replace_once(
    "README.md",
    "- 当前节点状态为 `VERIFYING`；clean PR canonical、squash merge 与 merged-stage canonical 尚未完成。未对用户现有 PostgreSQL 数据库执行写入/真实数据 sample 验收，也未宣称 Windows Full 人工交互验收完成。详细记录见 `docs/modular-rewrite/R06-entity-catalog-persistence/R06-02-team-names-and-profiles.md`。",
    "- clean PR canonical run `31898334326` / Windows job `95044999641` 为 `SUCCESS`；artifact `9250723192`，SHA-256 `15362b7164cc2ae0c7b177f4f4899b56678232595caa955d9602adea1eae2ee6`。PR #33 已按 fixed head `f04923425f9d660f2ce1275da2b689ca681f3500` squash merge 为 `334d5d86f08f5cd1adee5b23dc64d907aeb2eba2`。merged-stage canonical run `31925219003` / Windows job `95111615032` 为 `SUCCESS`；artifact `9257917686`，SHA-256 `ff29c7326ed23f3736fd433f2adde3aad587f9aa17506b2eedd96e913b7f2e9d`。R6-02 已正式关闭为 `DONE`，R6-03 Player Directory 与 Detail 开放为 `READY`。未对用户现有 PostgreSQL 数据库执行写入/真实数据 sample 验收，也未宣称 Windows Full 人工交互验收完成。详细记录见 `docs/modular-rewrite/R06-entity-catalog-persistence/R06-02-team-names-and-profiles.md`。",
)

# Stage index: advance only the completed node and its immediate successor.
replace_once(
    "docs/modular-rewrite/R06-entity-catalog-persistence/README.md",
    "| R6-02 | Team Names 与 Profiles | VERIFYING |\n| R6-03 | Player Directory 与 Detail | BLOCKED |",
    "| R6-02 | Team Names 与 Profiles | DONE |\n| R6-03 | Player Directory 与 Detail | READY |",
)
replace_once(
    "docs/modular-rewrite/R06-entity-catalog-persistence/README.md",
    "- 当前等待 clean PR canonical、squash merge 与 merged-stage canonical；R6-02 保持 `VERIFYING`，R6-03 继续 `BLOCKED`。",
    "- clean PR canonical run `31898334326` / Windows job `95044999641` 为 `SUCCESS`；artifact `9250723192`，SHA-256 `15362b7164cc2ae0c7b177f4f4899b56678232595caa955d9602adea1eae2ee6`。\n- PR #33 fixed head `f04923425f9d660f2ce1275da2b689ca681f3500` 已 squash merge 为 `334d5d86f08f5cd1adee5b23dc64d907aeb2eba2`。\n- merged-stage canonical run `31925219003` / Windows job `95111615032` 为 `SUCCESS`；artifact `9257917686`，SHA-256 `ff29c7326ed23f3736fd433f2adde3aad587f9aa17506b2eedd96e913b7f2e9d`。R6-02 正式 `DONE`，R6-03 开放为 `READY`。",
)
replace_once(
    "docs/modular-rewrite/R06-entity-catalog-persistence/README.md",
    "- R6-02 Team Names 与 Profiles 已进入 `VERIFYING`；当前仅剩 clean PR/merge/merged-stage canonical。\n- R6-03 继续 `BLOCKED`，直到 R6-02 完整收口为 `DONE`。",
    "- R6-02 Team Names 与 Profiles 已完成专项契约、阶段回归、clean PR、squash merge 与 merged-stage canonical，状态为 `DONE`。\n- R6-03 Player Directory 与 Detail 为当前唯一 `READY` 节点；R6-04～R6-10 继续 `BLOCKED`。",
)

# Node record: close the verification state without erasing earlier failures/corrections.
NODE = "docs/modular-rewrite/R06-entity-catalog-persistence/R06-02-team-names-and-profiles.md"
replace_once(NODE, "`VERIFYING`", "`DONE`")
replace_once(
    NODE,
    "Team Names 与 Team Profiles 的生产写入 owner 已从旧 `team_catalog.rs` 完整切换到 `adapters/catalog/teams/{names,profiles}/`；旧重复实现已删除，R6-09 删除职责继续留在原 owner。旧 owner 基线 PostgreSQL 契约、切换后专项契约、R6-01/R6-02 ownership、完整 architecture、模型保护、database baseline、frontend、rustfmt、workspace Clippy `-D warnings` 与 workspace tests 均已有实际成功证据。当前仅剩 clean PR canonical、squash merge 与 merged-stage canonical，因此不得提前标记为 `DONE`。",
    "Team Names 与 Team Profiles 的生产写入 owner 已从旧 `team_catalog.rs` 完整切换到 `adapters/catalog/teams/{names,profiles}/`；旧重复实现已删除，R6-09 删除职责继续留在原 owner。旧 owner 基线 PostgreSQL 契约、切换后专项契约、R6-01/R6-02 ownership、完整 architecture、模型保护、database baseline、frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests、clean PR canonical 与 merged-stage canonical 均已有实际成功证据。PR #33 已按固定 head squash merge，本节点正式 `DONE`。",
)
replace_once(
    NODE,
    "- 最终 hard-gate 验证源码 HEAD：`d9adf69892e271ef1b673aaea210c3873cbce570`。",
    "- 最终 hard-gate 验证源码 HEAD：`d9adf69892e271ef1b673aaea210c3873cbce570`。\n\n### clean PR 与 merged-stage canonical\n\n- clean PR fixed head：`f04923425f9d660f2ce1275da2b689ca681f3500`。Public Platform CI run `31898334326` / Windows job `95044999641`：`SUCCESS`；artifact `9250723192`，SHA-256 `15362b7164cc2ae0c7b177f4f4899b56678232595caa955d9602adea1eae2ee6`。\n- PR #33 使用 expected head `f04923425f9d660f2ce1275da2b689ca681f3500` squash merge；merge commit：`334d5d86f08f5cd1adee5b23dc64d907aeb2eba2`。\n- merged-stage Public Platform CI run `31925219003` / Windows job `95111615032`：`SUCCESS`；artifact `9257917686`，SHA-256 `ff29c7326ed23f3736fd433f2adde3aad587f9aa17506b2eedd96e913b7f2e9d`。",
)
replace_once(NODE, "## 当前净变更清单（clean PR 前）", "## 最终净变更清单")
replace_once(
    NODE,
    "## 未执行项与剩余门禁\n\n当前尚未宣称以下项目通过：\n\n- clean PR canonical Public Platform CI。\n- squash merge 与 merged-stage canonical Public Platform CI。\n- 用户现有 PostgreSQL 数据库写入/真实数据 sample 验收。\n- Windows Full 人工交互验收。\n\n前两项完成前 R6-02 只能保持 `VERIFYING`。后两项不被云端 Automated 替代，继续作为明确外部/人工限制保留。",
    "## 未执行项与剩余限制\n\nR6-02 节点要求的 clean PR、squash merge 与 merged-stage canonical 已全部完成。以下外部/人工验证仍未宣称执行：\n\n- 用户现有 PostgreSQL 数据库写入/真实数据 sample 验收。\n- Windows Full 人工交互验收。\n\n上述两项不被云端 Automated 替代；它们不阻塞本次纯持久化职责重写节点收口，但继续作为明确限制保留。",
)
replace_once(
    NODE,
    "- stage hard-gate verified HEAD：`d9adf69892e271ef1b673aaea210c3873cbce570`。\n- 回退使用 Git 提交恢复，不复制旧实现或保留长期兼容壳。",
    "- stage hard-gate verified HEAD：`d9adf69892e271ef1b673aaea210c3873cbce570`。\n- clean PR fixed head：`f04923425f9d660f2ce1275da2b689ca681f3500`。\n- R6-02 squash merge commit：`334d5d86f08f5cd1adee5b23dc64d907aeb2eba2`。\n- 回退使用 Git 提交恢复，不复制旧实现或保留长期兼容壳。",
)

# Transient closeout tooling must not survive in the final tree.
for transient in [
    ROOT / ".github/scripts/r6_02_closeout.py",
    ROOT / ".github/workflows/r6-02-closeout-docs.yml",
]:
    transient.unlink(missing_ok=False)
