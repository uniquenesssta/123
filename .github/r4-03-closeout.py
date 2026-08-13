from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8", newline="\n")


root_path = "README.md"
text = read(root_path)
lines = text.splitlines()
matches = [i for i, line in enumerate(lines) if line.startswith("- R4-03 Row 映射基础规范已建立 ")]
if len(matches) != 1:
    raise SystemExit(f"expected one root R4-03 record, found {len(matches)}")
lines[matches[0]] = (
    "- R4-03 Row 映射基础规范已建立 `mapping/time.rs`、`uuid.rs`、`json.rs`、`optional.rs`、`invalid_state.rs` 五类基础标量职责；"
    "共享 CompetitionKind 解析迁入独立 `competition_kind.rs`，根模块仅保留兼容 re-export，未建立万能动态 mapper。"
    "strict hard gate run `31674700550` 已通过 mapping 专项、R4-01/R4-02 回归、数据库冻结/保护资产、完整 architecture/frontend、Persistence check/tests、workspace Clippy `-D warnings` 与 workspace tests；"
    "PR #24 clean Public Platform CI run `31675727990` / job `94369762967` 为 `SUCCESS`，PR 按固定 HEAD `202648df6aa1f9a14eb03bdcabcbd5ee0a271e56` 合并，merge commit `0e5a68e09c6c06204b926f4d30c45262740d983b`。"
    "合并后 stage Public Platform CI run `31677600876` / job `94375513281` 同样为 `SUCCESS`；artifact `9172855279` 大小 `13909093` 字节，SHA-256 `31719ff04f00eb944c84fcd37dbbda3fa6252f7d55bc42a1a8af3d903cad3544`。"
    "18 个专用 PostgreSQL 集成测试仍未执行，未执行 destructive database reset。R4-03 状态正式关闭为 `DONE`，R4-04 已开放为 `READY`。"
)
write(root_path, "\n".join(lines) + "\n")

stage_path = "docs/modular-rewrite/R04-persistence-foundation/README.md"
text = read(stage_path)
old_r403 = "| R4-03 | 通用 Row 映射基础规范 | VERIFYING | [`R04-03-row-映射基础规范.md`](./R04-03-row-映射基础规范.md) |"
new_r403 = "| R4-03 | 通用 Row 映射基础规范 | DONE | [`R04-03-row-映射基础规范.md`](./R04-03-row-映射基础规范.md) |"
old_r404 = "| R4-04 | Application Port Adapter 注册 | BLOCKED | — |"
new_r404 = "| R4-04 | Application Port Adapter 注册 | READY | — |"
if text.count(old_r403) != 1 or text.count(old_r404) != 1:
    raise SystemExit("R4 task table state mismatch before closeout")
text = text.replace(old_r403, new_r403, 1).replace(old_r404, new_r404, 1)
marker = "\n## R4-03 实施中\n"
if text.count(marker) != 1:
    raise SystemExit(f"expected one R4-03 implementation section, found {text.count(marker)}")
prefix = text.split(marker, 1)[0].rstrip()
closeout = """

## R4-03 收口

- 从 R4-02 正式收口 HEAD `3c147376cf81394984cc20850a58e866eef4280b` 独立建立 `agent/r4-03-row-mapping`；最终实现提交为 `202648df6aa1f9a14eb03bdcabcbd5ee0a271e56`。
- strict hard gate run `31674700550` 已通过 R4-03 mapping 专项、R4-01/R4-02 Persistence/Audit 回归、数据库冻结与保护资产、rustfmt、Persistence check/tests、完整 architecture/frontend、workspace Clippy `-D warnings` 与 workspace tests；最终实现树严格为 15 个目标文件，临时 workflow/helper 与额外 untracked 均为 0。
- PR #24 clean Public Platform CI run `31675727990` / Windows automated delivery job `94369762967`：`SUCCESS`；PR 按固定 HEAD `202648df6aa1f9a14eb03bdcabcbd5ee0a271e56` 合并到 `rewrite/r4-persistence-foundation`，merge commit `0e5a68e09c6c06204b926f4d30c45262740d983b`。
- 合并后 stage Public Platform CI run `31677600876` / job `94375513281`：`SUCCESS`；artifact `9172855279`（`windows-automated-delivery-evidence-0e5a68e09c6c06204b926f4d30c45262740d983b`）大小 `13909093` 字节，SHA-256 `31719ff04f00eb944c84fcd37dbbda3fa6252f7d55bc42a1a8af3d903cad3544`。
- 初次 formal-closeout workflow run `31679755024` 因内嵌 Python 多行文本破坏 YAML block 缩进而在调度前失败（0 job）；未执行文档修改、未产生 closeout commit、未改变生产源码。恢复收口改用独立临时 helper，最终提交前 workflow/helper 均自删除。
- 18 个要求专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试仍未执行；未执行 destructive database reset。
- R4-03 正式关闭为 `DONE`；R4-04 开放为 `READY`。本收口未包含任何 R4-04 生产源码改动。
"""
write(stage_path, prefix + closeout + "\n")

task_path = "docs/modular-rewrite/R04-persistence-foundation/R04-03-row-映射基础规范.md"
text = read(task_path)
if text.count("`VERIFYING`") != 1:
    raise SystemExit(f"expected one VERIFYING task status, found {text.count('`VERIFYING`')}")
text = text.replace("`VERIFYING`", "`DONE`", 1)
text = text.replace(
    "- 无生产文件删除。两份临时 R4-03 执行文件在最终实施提交前自删除，不进入最终源码树。",
    "- 无生产文件删除。临时 R4-03 workflow/helper 在最终实施提交前全部自删除，不进入最终源码树。",
    1,
)
compat_old = "- 本节点保持 `VERIFYING`，等待 clean Public Platform CI 与正式 PR 合并；R4-04 继续 `BLOCKED`。"
compat_new = "- 本节点已正式关闭为 `DONE`；R4-04 已开放为 `READY`，但本节点未提前实施任何 R4-04 生产源码。"
if text.count(compat_old) != 1:
    raise SystemExit("R4-03 compatibility status marker missing")
text = text.replace(compat_old, compat_new, 1)
section_marker = "\n## 兼容性与剩余风险\n"
if text.count(section_marker) != 1:
    raise SystemExit("R4-03 compatibility section marker mismatch")
formal = """
## 正式收口

- 最终实现提交：`202648df6aa1f9a14eb03bdcabcbd5ee0a271e56`；相对 R4-03 基线最终仅包含 15 个目标实现/验证/文档文件，临时 workflow/helper 未进入最终树。
- PR #24 clean Public Platform CI run `31675727990` / job `94369762967`：`SUCCESS`，随后 PR 按固定 HEAD 合并；merge commit：`0e5a68e09c6c06204b926f4d30c45262740d983b`。
- 合并后 stage Public Platform CI run `31677600876` / job `94375513281`：`SUCCESS`；validation evidence artifact `9172855279`，大小 `13909093` 字节，SHA-256 `31719ff04f00eb944c84fcd37dbbda3fa6252f7d55bc42a1a8af3d903cad3544`。
- 初次 formal-closeout workflow run `31679755024`：YAML 在调度前解析失败，0 job；未修改三份 canonical 文档、未产生 closeout commit、未改变任何生产源码。恢复路径使用独立临时 helper，并要求最终树对 `.github/workflows/r4-03-closeout.yml` 与 `.github/r4-03-closeout.py` 均为零差异。
- 未执行项保持不变：18 个要求专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未执行；未执行 destructive database reset。

"""
text = text.replace(section_marker, "\n" + formal + "## 兼容性与剩余风险\n", 1)
write(task_path, text)
