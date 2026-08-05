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


source_path = "scripts/task-ui-screenshot-tools.mjs"
source = read(source_path)
old_helper_anchor = """function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function requestJson({ port, path: requestPath, method = \"GET\" }) {
"""
new_helper_anchor = """function sleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function readDevToolsPort(portFile, browserStderr) {
  let lastError = null;
  for (let attempt = 0; attempt < 150; attempt += 1) {
    try {
      const [portText] = fs.readFileSync(portFile, \"utf8\").trim().split(/\\s+/);
      const port = Number(portText);
      if (Number.isInteger(port) && port > 0) return port;
      lastError = new Error(`调试端口内容无效：${portText || \"empty\"}`);
    } catch (error) {
      const code = error && typeof error === \"object\" && \"code\" in error ? error.code : null;
      if (![\"EBUSY\", \"ENOENT\", \"EPERM\"].includes(code)) throw error;
      lastError = error;
    }
    await sleep(100);
  }

  const detail = lastError instanceof Error ? `${lastError.code ?? lastError.name}: ${lastError.message}` : String(lastError ?? \"unknown\");
  throw new Error(`浏览器未开放可读调试端口（${detail}）。${browserStderr().slice(-1200)}`);
}

function requestJson({ port, path: requestPath, method = \"GET\" }) {
"""
source = replace_once(source, old_helper_anchor, new_helper_anchor, "insert readDevToolsPort")
old_usage = """  try {
    for (let attempt = 0; attempt < 150 && !fs.existsSync(portFile); attempt += 1) await sleep(100);
    if (!fs.existsSync(portFile)) throw new Error(`浏览器未开放调试端口。${stderr.slice(-1200)}`);
    const [portText] = fs.readFileSync(portFile, \"utf8\").trim().split(/\\s+/);
    const port = Number(portText);
"""
new_usage = """  try {
    const port = await readDevToolsPort(portFile, () => stderr);
"""
source = replace_once(source, old_usage, new_usage, "replace DevToolsActivePort read")
write(source_path, source)

record_path = "docs/modular-rewrite/R01-architecture-composition/R01-02-边界验证脚本.md"
record = read(record_path)
old_pending = """## 8. 待执行门禁

- 最终 `new-B` HEAD 的 `Public Platform CI` Windows Automated 全链路。
- 通过前本节点保持 `VERIFYING`，R1-03 继续 `BLOCKED`。
- 真实 PostgreSQL、Windows Full 和用户本机实机验收仍按阶段规则留到最终统一验收。
"""
new_pending = """## 8. Windows 门禁过程与待执行项

- 正式 workflow run `31000821757` 的独立 `npm run verify:architecture` 以及前端聚合中的三条架构门禁均通过。
- 同一运行随后在任务 UI 截图启动 Chromium 时失败：Windows 对已创建的 `DevToolsActivePort` 返回瞬时 `EBUSY`；失败发生在截图基础设施，尚未进入 Rust、Tauri release 与客户端启动阶段，因此本节点不能关闭。
- `task-ui-screenshot-tools.mjs` 已将“文件存在后单次读取”改为最长 15 秒的“存在、可读、端口有效”有界重试；只重试 `EBUSY`、`ENOENT`、`EPERM` 和未完成端口内容，其他错误立即抛出，超时仍硬失败。
- 待最终 `new-B` HEAD 的 `Public Platform CI` Windows Automated 全链路通过后，才可把 R1-02 标记为 `DONE` 并开放 R1-03。
- 真实 PostgreSQL、Windows Full 和用户本机实机验收仍按阶段规则留到最终统一验收。
"""
record = replace_once(record, old_pending, new_pending, "update R1-02 validation record")
write(record_path, record)

readme_path = "README.md"
readme = read(readme_path)
anchor = "- R1-02 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，接入 `npm run verify:architecture` 与前端聚合验证；当前等待 Windows CI 独立步骤接入和最终 `new-B` HEAD 的 Windows Automated 结果，状态为 `VERIFYING`。"
replacement = "- R1-02 已新增模块边界、状态所有权和受保护导入三条仓库内门禁，接入 `npm run verify:architecture`、前端聚合验证和 Windows CI 独立步骤；首轮正式 Windows 运行的架构门禁通过，随后因 Chromium `DevToolsActivePort` 瞬时 `EBUSY` 停止，现已增加最长 15 秒的可读端口有界重试，状态保持 `VERIFYING`。"
readme = replace_once(readme, anchor, replacement, "update root README R1-02 line")
write(readme_path, readme)

subprocess.run(["node", "--check", source_path], cwd=ROOT, check=True)
subprocess.run(["git", "config", "user.name", "uniquenesssta"], cwd=ROOT, check=True)
subprocess.run(["git", "config", "user.email", "uniquenesssta@live.com"], cwd=ROOT, check=True)
subprocess.run(["git", "config", "core.quotepath", "false"], cwd=ROOT, check=True)
subprocess.run(["git", "add", source_path, record_path, readme_path], cwd=ROOT, check=True)
changed = subprocess.check_output(["git", "diff", "--cached", "--name-only"], cwd=ROOT, text=True).splitlines()
expected = sorted([source_path, record_path, readme_path])
if sorted(changed) != expected:
    raise RuntimeError(f"unexpected staged scope: {changed}")
subprocess.run(["git", "commit", "-m", "fix(ci): retry locked Chromium debug port"], cwd=ROOT, check=True)
subprocess.run(["git", "push", "origin", "HEAD:new-B"], cwd=ROOT, check=True)
