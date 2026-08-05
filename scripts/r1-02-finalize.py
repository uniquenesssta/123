from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UTF8 = "utf-8"


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding=UTF8).replace("\r\n", "\n")


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content.replace("\r\n", "\n"), encoding=UTF8, newline="\n")


def replace_once(content: str, old: str, new: str, label: str) -> str:
    count = content.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected exactly one match, found {count}")
    return content.replace(old, new, 1)


def run(*args: str) -> None:
    subprocess.run(args, cwd=ROOT, check=True)


record_path = "docs/modular-rewrite/R01-architecture-composition/R01-02-边界验证脚本.md"
record = read(record_path)
record = replace_once(
    record,
    "- 任务状态：`VERIFYING`",
    "- 任务状态：`DONE`",
    "R1-02 task status",
)
old_section = """## 8. Windows 门禁过程与待执行项

- 正式 workflow run `31000821757` 的独立 `npm run verify:architecture` 以及前端聚合中的三条架构门禁均通过。
- 同一运行随后在任务 UI 截图启动 Chromium 时失败：Windows 对已创建的 `DevToolsActivePort` 返回瞬时 `EBUSY`；失败发生在截图基础设施，尚未进入 Rust、Tauri release 与客户端启动阶段，因此本节点不能关闭。
- `task-ui-screenshot-tools.mjs` 已将“文件存在后单次读取”改为最长 15 秒的“存在、可读、端口有效”有界重试；只重试 `EBUSY`、`ENOENT`、`EPERM` 和未完成端口内容，其他错误立即抛出，超时仍硬失败。
- 待最终 `new-B` HEAD 的 `Public Platform CI` Windows Automated 全链路通过后，才可把 R1-02 标记为 `DONE` 并开放 R1-03。
- 真实 PostgreSQL、Windows Full 和用户本机实机验收仍按阶段规则留到最终统一验收。
"""
new_section = """## 8. Windows 门禁过程与最终结果

- 首轮正式 workflow run `31000821757` 的独立 `npm run verify:architecture` 以及前端聚合中的三条架构门禁均通过；随后因 Windows 对 Chromium `DevToolsActivePort` 返回瞬时 `EBUSY` 而停止。
- `task-ui-screenshot-tools.mjs` 将“文件存在后单次读取”改为最长 15 秒的“存在、可读、端口有效”有界重试；只重试 `EBUSY`、`ENOENT`、`EPERM` 和未完成端口内容，其他错误立即抛出，超时仍硬失败。未放宽截图差异阈值、未跳过截图门禁。
- 最终候选提交 `28ec363babe4f3fbccd14693d0261febdc305458` 的 `Public Platform CI` workflow run `31001470224`、job `92291121763` 为 `success`。
- 独立架构门禁、前端契约/类型/截图/生产构建、Rust 格式/Clippy/工作区测试、Tauri Windows release、release 客户端启动和运行日志扫描全部通过。
- Windows Automated 报告状态为 `pass`，应用版本 `0.23.0`，7 条日志记录、0 条无效记录、0 个运行时错误；完成操作包括 `bootstrap`、`read_workspace_state`、`save_workspace_state`。
- 验证 artifact `8929207011` 名称为 `windows-automated-delivery-evidence-28ec363babe4f3fbccd14693d0261febdc305458`，大小 `14117150` 字节，SHA-256 为 `e83b2ab9c6cb705d0bfd740c798673a45dc2a4cb0b7b35ddebe844bb40b13e88`，保留至 2026-08-19。
- R1-02 状态关闭为 `DONE`，R1-03 开放为 `READY`；R1-04 与 R1-05 继续 `BLOCKED`。
- 真实 PostgreSQL、Windows Full 和用户本机实机验收仍按阶段规则留到最终统一验收。
"""
record = replace_once(record, old_section, new_section, "R1-02 final validation section")
write(record_path, record)

stage_path = "docs/modular-rewrite/R01-architecture-composition/README.md"
stage = read(stage_path)
stage = replace_once(
    stage,
    "| R1-02 | 边界验证脚本 | VERIFYING | [`R01-02-边界验证脚本.md`](R01-02-边界验证脚本.md) | `npm run verify:architecture` 通过 | Windows Automated 待最终 HEAD 验证 |",
    "| R1-02 | 边界验证脚本 | DONE | [`R01-02-边界验证脚本.md`](R01-02-边界验证脚本.md) | `npm run verify:architecture` 通过 | workflow run `31001470224` 通过 |",
    "stage R1-02 row",
)
stage = replace_once(
    stage,
    "| R1-03 | 浏览器组合根 | BLOCKED | 待创建 | 待执行 | 待执行 |",
    "| R1-03 | 浏览器组合根 | READY | 待创建 | 待执行 | 待执行 |",
    "stage R1-03 row",
)
old_stage_result = """## R1-02 当前结果

- 已新增模块边界、状态所有权和受保护导入三条门禁，并接入 `verify:frontend`；Windows CI 独立步骤待受控提交。
- 当前状态为 `VERIFYING`；完整 Windows Automated 通过后才可开放 R1-03。

## 当前唯一可执行任务

`R1-02 边界验证脚本：完成 CI 接入与最终 Windows 自动化门禁并关闭节点`
"""
new_stage_result = """## R1-02 完成结果

- 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，共用独立的文件遍历、Import 解析、Cargo 解析和报告模块。
- 两份 R1 架构契约状态均为 `ACTIVE`；`npm run verify:architecture`、前端聚合验证和 Windows CI 独立步骤均已接入。
- 三条现存耦合只按精确源/目标与退出任务登记：Application→PostgreSQL 在 `R1-05` 退出，Prediction→P4 Workbench 在 `R14-06` 退出，Prediction→Runs 在 `R14-07` 退出。
- workflow run `31001470224` 在提交 `28ec363babe4f3fbccd14693d0261febdc305458` 上完整通过；artifact `8929207011` 的 SHA-256 为 `e83b2ab9c6cb705d0bfd740c798673a45dc2a4cb0b7b35ddebe844bb40b13e88`。
- R1-02 状态为 `DONE`；R1-03 开放为 `READY`，R1-04 与 R1-05 继续 `BLOCKED`。

## 当前唯一可执行任务

`R1-03 浏览器组合根`
"""
stage = replace_once(stage, old_stage_result, new_stage_result, "stage R1-02 result block")
write(stage_path, stage)

root_path = "README.md"
root = read(root_path)
old_root_line = "- R1-02 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，接入 `npm run verify:architecture`、前端聚合验证和 Windows CI 独立步骤；首轮正式 Windows 运行的架构门禁通过，随后因 Chromium `DevToolsActivePort` 瞬时 `EBUSY` 停止，现已增加最长 15 秒的可读端口有界重试，状态保持 `VERIFYING`。"
new_root_lines = """- R1-02 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，接入 `npm run verify:architecture`、前端聚合验证和 Windows CI 独立步骤；状态为 `DONE`，R1-03 已开放为 `READY`。
- R1-02 最终 workflow run `31001470224`、job `92291121763` 在提交 `28ec363babe4f3fbccd14693d0261febdc305458` 上通过；artifact `8929207011` 大小 `14117150` 字节，SHA-256 为 `e83b2ab9c6cb705d0bfd740c798673a45dc2a4cb0b7b35ddebe844bb40b13e88`，Automated 报告为 PASS，7 条运行记录、0 条无效记录、0 个运行时错误。
- 截图启动工具仅对 Chromium `DevToolsActivePort` 的 `EBUSY`、`ENOENT`、`EPERM` 和未完成端口内容执行最长 15 秒的有界重试；其他错误立即失败，截图差异阈值与门禁强度未放宽。"""
root = replace_once(root, old_root_line, new_root_lines, "root README R1-02 status")
write(root_path, root)

script_path = ROOT / "scripts/r1-02-finalize.py"
script_path.unlink()

run("git", "config", "user.name", "uniquenesssta")
run("git", "config", "user.email", "uniquenesssta@live.com")
run("git", "config", "core.quotepath", "false")
run("git", "add", "-A")
changed = subprocess.check_output(
    ["git", "diff", "--cached", "--name-only"],
    cwd=ROOT,
    text=True,
    encoding="utf-8",
).splitlines()
expected = sorted([
    record_path,
    stage_path,
    root_path,
    "scripts/r1-02-finalize.py",
])
if sorted(changed) != expected:
    raise RuntimeError(f"unexpected staged scope: {changed}")

run("git", "commit", "-m", "docs(r1): close R1-02 boundary gates")
run("git", "push", "origin", "HEAD:new-B")
