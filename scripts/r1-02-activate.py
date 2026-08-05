from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
UTF8 = "utf-8"


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding=UTF8).replace("\r\n", "\n")


def write(path: str, content: str) -> None:
    (ROOT / path).write_text(content.replace("\r\n", "\n"), encoding=UTF8, newline="\n")


def run(*args: str) -> None:
    subprocess.run(args, cwd=ROOT, check=True)


module_path = "architecture/module-boundaries.json"
module = json.loads(read(module_path))
module["status"] = "ACTIVE"
module.setdefault("policy", {})["validators"] = {
    "module_boundaries": "scripts/architecture/verifyModuleBoundaries.mjs",
    "state_ownership": "scripts/architecture/verifyStateOwnership.mjs",
    "protected_imports": "scripts/architecture/verifyProtectedImports.mjs",
}
transition = {
    "from": "football-application",
    "to": "football-persistence-postgres",
    "reason": "existing application persistence coupling retained until the application composition task",
    "exit_task": "R1-05",
}
module["transitional_edges"] = [
    edge
    for edge in module.get("transitional_edges", [])
    if not (edge.get("from") == transition["from"] and edge.get("to") == transition["to"])
]
module["transitional_edges"].append(transition)
write(module_path, json.dumps(module, ensure_ascii=False, indent=2) + "\n")

state_path = "architecture/state-ownership.json"
state = json.loads(read(state_path))
state["status"] = "ACTIVE"
state.setdefault("policy", {})["validator"] = "scripts/architecture/verifyStateOwnership.mjs"
write(state_path, json.dumps(state, ensure_ascii=False, indent=2) + "\n")

package_path = "package.json"
package = json.loads(read(package_path))
expected_architecture_command = (
    "node scripts/architecture/verifyModuleBoundaries.mjs && "
    "node scripts/architecture/verifyStateOwnership.mjs && "
    "node scripts/architecture/verifyProtectedImports.mjs"
)
updated_scripts: dict[str, str] = {}
for key, value in package["scripts"].items():
    updated_scripts[key] = value
    if key == "verify:frontend":
        updated_scripts["verify:architecture"] = expected_architecture_command
package["scripts"] = updated_scripts
write(package_path, json.dumps(package, ensure_ascii=False, indent=2) + "\n")

frontend_path = "scripts/verify-frontend.mjs"
frontend = read(frontend_path)
architecture_checks = (
    '  "architecture/verifyModuleBoundaries.mjs",\n'
    '  "architecture/verifyStateOwnership.mjs",\n'
    '  "architecture/verifyProtectedImports.mjs",\n'
)
marker = "const nodeChecks = [\n"
if architecture_checks not in frontend:
    if marker not in frontend:
        raise RuntimeError("verify-frontend.mjs nodeChecks marker not found")
    frontend = frontend.replace(marker, marker + architecture_checks, 1)
write(frontend_path, frontend)

ci_path = ".github/workflows/ci.yml"
ci = read(ci_path)
start = ci.find("      - name: Verify R1 architecture contract JSON\n")
end_marker = "      - name: Run Windows automated acceptance\n"
end = ci.find(end_marker)
if start < 0 or end < 0 or end <= start:
    raise RuntimeError("CI architecture smoke-check block not found")
replacement = (
    "      - name: Verify R1 architecture boundaries\n"
    "        shell: pwsh\n"
    "        working-directory: 项目源码\n"
    "        run: npm run verify:architecture\n\n"
)
ci = ci[:start] + replacement + ci[end:]
write(ci_path, ci)

record_path = "docs/modular-rewrite/R01-architecture-composition/R01-02-边界验证脚本.md"
record = """# R01-02 边界验证脚本：实施记录

## 1. 基本信息

- 所属阶段：R01 架构契约与空壳组合根
- 任务状态：`VERIFYING`
- 实施分支：`new-B`
- 开始基线：`9f4a3c9aa3351733500841afd4b4d5a85c417ba3`
- 实施日期：2026-08-05
- 对应任务书：`docs/football-model-platform-modular-rewrite-19-docs/01-R1-architecture-composition.md`

## 2. 目标与实际问题

R1-01 已建立机器可读的模块边界和状态所有权契约，但当时只有 CI 内联 smoke check，不能持续扫描真实 TypeScript Import、Cargo workspace 依赖、SQLx/Tauri 使用范围、受保护模型导入及状态 owner 源位置。本节点把这些职责实现为仓库内可本地运行、可被 Windows CI 重复执行的非零退出码门禁。

## 3. 实际实现

- `verifyModuleBoundaries.mjs`：验证 Feature owner、跨 Feature 内部导入、前端禁止路径、Cargo workspace 成员、crate 允许依赖和依赖环。
- `verifyStateOwnership.mjs`：验证状态数量、唯一 id、当前 owner、writer/forbidden 声明、owner 文件和关键符号存在性。
- `verifyProtectedImports.mjs`：限制 `@tauri-apps/api/core`、SQLx、Tauri 和 P4/P7 受保护模型导入范围，并验证 Domain 不依赖基础设施。
- 公共文件遍历、JavaScript Import 解析、Cargo 解析和报告输出分别位于 `scripts/architecture/lib/`，三条门禁不复制实现。
- `package.json` 新增 `verify:architecture`，`verify-frontend.mjs` 和 Windows CI 均接入三条门禁。
- 两份 R1 架构契约状态切换为 `ACTIVE` 并登记验证器路径。

## 4. 受控过渡边

任务书目标要求 `football-application` 不依赖 `football-persistence-postgres`，但 R1-01 冻结的当前 Cargo 结构仍存在该直接依赖；本节点不得跨范围实施 R1-05 的 Application 组合根重构。因此该边被明确登记为仅到 `R1-05` 退出的受控过渡边。验证器只接受契约中明确登记的这一条过渡边，不将其描述为目标架构已完成。

## 5. 文件清单

### 新增

- `scripts/architecture/lib/repository.mjs`
- `scripts/architecture/lib/imports.mjs`
- `scripts/architecture/lib/cargo.mjs`
- `scripts/architecture/lib/report.mjs`
- `scripts/architecture/verifyModuleBoundaries.mjs`
- `scripts/architecture/verifyStateOwnership.mjs`
- `scripts/architecture/verifyProtectedImports.mjs`
- `docs/modular-rewrite/R01-architecture-composition/R01-02-边界验证脚本.md`

### 修改

- `architecture/module-boundaries.json`
- `architecture/state-ownership.json`
- `package.json`
- `scripts/verify-frontend.mjs`
- `.github/workflows/ci.yml`
- `docs/modular-rewrite/R01-architecture-composition/README.md`
- `README.md`

### 移动、重命名、删除

无永久文件移动、重命名或删除。

## 6. 兼容性

- 未修改公共 Tauri 命令、DTO、Schema、数据库格式、配置键、错误语义、日志等级、UI 或模型保护资产。
- 未新增生产或开发依赖，未修改 `package-lock.json`、`Cargo.lock` 或迁移。
- 验证器只读取仓库文件并输出诊断；不写入业务状态，不执行数据库、网络或应用运行时副作用。

## 7. 已执行验证

| 验证 | 环境 | 结果 |
|---|---|---|
| 七个新增 `.mjs` 的 `node --check` | 隔离容器 Node.js | 通过 |
| `npm run verify:architecture` | GitHub Actions 激活任务工作树 | 通过后才允许提交激活变更 |

## 8. 待执行门禁

- 最终 `new-B` HEAD 的 `Public Platform CI` Windows Automated 全链路。
- 通过前本节点保持 `VERIFYING`，R1-03 继续 `BLOCKED`。
- 真实 PostgreSQL、Windows Full 和用户本机实机验收仍按阶段规则留到最终统一验收。

## 9. 回退

回退到 R1-02 开始基线 `9f4a3c9aa3351733500841afd4b4d5a85c417ba3`。本节点没有数据迁移、依赖升级或公共接口迁移，不需要额外数据回滚。
"""
write(record_path, record)

stage_path = "docs/modular-rewrite/R01-architecture-composition/README.md"
stage = read(stage_path)
stage = stage.replace(
    "| R1-02 | 边界验证脚本 | READY | 待创建 | 待执行 | 待执行 |",
    "| R1-02 | 边界验证脚本 | VERIFYING | [`R01-02-边界验证脚本.md`](R01-02-边界验证脚本.md) | `npm run verify:architecture` 通过 | Windows Automated 待最终 HEAD 验证 |",
)
stage = stage.replace(
    "## 当前唯一可执行任务\n\n`R1-02 边界验证脚本`",
    "## R1-02 当前结果\n\n- 已新增模块边界、状态所有权和受保护导入三条门禁，并接入 `verify:frontend` 与 Windows CI。\n- 当前状态为 `VERIFYING`；完整 Windows Automated 通过后才可开放 R1-03。\n\n## 当前唯一可执行任务\n\n`R1-02 边界验证脚本：完成最终 Windows 自动化门禁并关闭节点`",
)
write(stage_path, stage)

root_readme_path = "README.md"
root_readme = read(root_readme_path)
r1_01_line = "- `new-B` 已从 `new-A` 提交 `36d34ba1ff73cbec575cf58594aa8c0329669496` 建立；R1-01 已创建模块边界与状态所有权契约并完成 Windows 自动化门禁，状态为 `DONE`，R1-02 已开放为 `READY`。"
r1_02_line = "- R1-02 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，接入 `npm run verify:architecture`、前端聚合验证和 Windows CI；当前等待最终 `new-B` HEAD 的 Windows Automated 结果，状态为 `VERIFYING`。"
if r1_01_line not in root_readme:
    raise RuntimeError("Root README R1-01 line not found")
root_readme = root_readme.replace(r1_01_line, r1_01_line + "\n" + r1_02_line, 1)
write(root_readme_path, root_readme)

for script in [
    "scripts/architecture/lib/repository.mjs",
    "scripts/architecture/lib/imports.mjs",
    "scripts/architecture/lib/cargo.mjs",
    "scripts/architecture/lib/report.mjs",
    "scripts/architecture/verifyModuleBoundaries.mjs",
    "scripts/architecture/verifyStateOwnership.mjs",
    "scripts/architecture/verifyProtectedImports.mjs",
]:
    run("node", "--check", script)
run("npm", "run", "verify:architecture")

workflow_path = ROOT / ".github/workflows/r1-02-activate.yml"
activation_script_path = ROOT / "scripts/r1-02-activate.py"
workflow_path.unlink()
activation_script_path.unlink()

run("git", "config", "user.name", "uniquenesssta")
run("git", "config", "user.email", "uniquenesssta@live.com")
run("git", "config", "core.quotepath", "false")
run("git", "add", "-A")
changed = subprocess.check_output(["git", "diff", "--cached", "--name-only"], cwd=ROOT, text=True).splitlines()
required = {
    "architecture/module-boundaries.json",
    "architecture/state-ownership.json",
    "package.json",
    "scripts/verify-frontend.mjs",
    ".github/workflows/ci.yml",
    "docs/modular-rewrite/R01-architecture-composition/R01-02-边界验证脚本.md",
    "docs/modular-rewrite/R01-architecture-composition/README.md",
    "README.md",
    ".github/workflows/r1-02-activate.yml",
    "scripts/r1-02-activate.py",
}
missing = sorted(required.difference(changed))
if missing:
    raise RuntimeError(f"Expected activation changes missing: {missing}")

run("git", "commit", "-m", "feat(r1): activate architecture boundary gates")
run("git", "push", "origin", "HEAD:new-B")
