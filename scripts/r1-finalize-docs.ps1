$ErrorActionPreference = "Stop"
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)

function Read-RepoText([string]$Path) {
  return [IO.File]::ReadAllText($Path).Replace("`r`n", "`n")
}

function Write-RepoText([string]$Path, [string]$Content) {
  [IO.File]::WriteAllText($Path, $Content.Replace("`r`n", "`n"), $utf8NoBom)
}

$rootPath = "README.md"
$root = Read-RepoText $rootPath
$oldRootLine = '- `new-B` 已从 `new-A` 提交 `36d34ba1ff73cbec575cf58594aa8c0329669496` 建立；R1-01 已创建模块边界与状态所有权契约，当前因完整工作树门禁未执行保持 `VERIFYING`，未修改业务源码、依赖、迁移、公共命令或模型保护资产。'
$newRootBlock = @'
- `new-B` 已从 `new-A` 提交 `36d34ba1ff73cbec575cf58594aa8c0329669496` 建立；R1-01 已创建模块边界与状态所有权契约并完成 Windows 自动化门禁，状态为 `DONE`，R1-02 已开放为 `READY`。
- `Public Platform CI` 现支持推送到 `main`、`new-*`、`rewrite/**`、Pull Request 和 `workflow_dispatch`，以 `windows-latest` 执行架构契约、前端、Rust、Tauri Windows release、release 客户端启动和运行日志扫描，并上传验证证据。
- R1-01 验证运行 `30989439570`、job `92251837163` 在提交 `fc02ad51d01229cb2ea62fc20f623910ba49de7f` 上通过；artifact `8924033934` 大小 `14115361` 字节，SHA-256 为 `85551aacdd43ba1e3516025ae510aefaaa8e11d61f433a701eaa884e292a47a1`，Automated 报告为 PASS，7 条运行记录、0 条无效记录、0 个运行时错误。
- 新增 `.gitattributes` 固定文本 LF 和二进制排除规则；相关验证器统一按 LF 规范读取冻结合同，避免 Windows 检出换行导致伪失败。冻结合同、迁移哈希、锁文件、生产依赖、公共命令、数据库结构和模型保护资产均未改变。
'@.Trim()
if (-not $root.Contains($oldRootLine)) {
  throw "Root README R1-01 status line was not found."
}
$root = $root.Replace($oldRootLine, $newRootBlock)
$marker = "`n`n## 模块化重写执行记录"
$ciParagraph = @'

`Public Platform CI` 是 Windows 自动交付门禁：对 `main`、`new-*`、`rewrite/**` 的推送、Pull Request 和手动触发执行 R1 架构契约检查及 `scripts/windows-acceptance.ps1 -Mode Automated`，并保存验收日志和 release bundle 证据。云端 Automated 不替代最终真实 PostgreSQL、Windows Full 交互和用户本机验收。

## 模块化重写执行记录
'@
if (-not $root.Contains($marker)) {
  throw "Root README modular rewrite marker was not found."
}
$root = $root.Replace($marker, $ciParagraph)
Write-RepoText $rootPath $root

$stagePath = "docs/modular-rewrite/R01-architecture-composition/README.md"
$stage = Read-RepoText $stagePath
$stage = $stage.Replace(
  '| R1-01 | 模块边界契约 | VERIFYING | [`R01-01-模块边界契约.md`](R01-01-模块边界契约.md) | JSON 解析与契约自检通过 | 完整工作树门禁待执行 |',
  '| R1-01 | 模块边界契约 | DONE | [`R01-01-模块边界契约.md`](R01-01-模块边界契约.md) | JSON 解析、契约自检、Windows Automated 通过 | workflow run `30989439570` 通过 |'
)
$stage = $stage.Replace(
  '| R1-02 | 边界验证脚本 | BLOCKED | 待创建 | 待执行 | 待执行 |',
  '| R1-02 | 边界验证脚本 | READY | 待创建 | 待执行 | 待执行 |'
)
$stage = $stage.Replace('## R1-01 当前结果', '## R1-01 完成结果')
$oldStageTail = '- 因当前执行环境不能建立完整 Git 工作树，`npm run verify:frontend`、Rust 全门禁和现有架构脚本尚未在本提交执行；R1-01 保持 `VERIFYING`，R1-02 不开放。'
$newStageTail = @'
- 已将 `Public Platform CI` 扩展为 Windows 自动交付门禁，覆盖目标分支推送、PR 和手动触发。
- workflow run `30989439570`、job `92251837163` 在提交 `fc02ad51d01229cb2ea62fc20f623910ba49de7f` 上通过；证据 artifact `8924033934` 的 SHA-256 为 `85551aacdd43ba1e3516025ae510aefaaa8e11d61f433a701eaa884e292a47a1`。
- 前端、Rust、Tauri Windows release、release 客户端启动和运行日志扫描均通过；真实 PostgreSQL、Windows Full 和用户本机验收仍保留到最终统一验收。
- R1-01 状态为 `DONE`，R1-02 开放为 `READY`；R1-03 至 R1-05 继续 `BLOCKED`。
'@.Trim()
if (-not $stage.Contains($oldStageTail)) {
  throw "Stage README verification tail was not found."
}
$stage = $stage.Replace($oldStageTail, $newStageTail)
$stage = $stage.Replace(
  '`R1-01 模块边界契约：完成完整工作树验证并关闭节点`',
  '`R1-02 边界验证脚本`'
)
Write-RepoText $stagePath $stage

$taskPath = "docs/modular-rewrite/R01-architecture-composition/R01-01-模块边界契约.md"
$task = Read-RepoText $taskPath
$task = $task.Replace('- 任务状态：`VERIFYING`', '- 任务状态：`DONE`')
$task = $task.Replace(
  '- 更新根 `README.md` 和 R01 阶段索引，准确标记任务已实施但完整门禁仍待执行。',
  '- 更新 `.github/workflows/ci.yml`，将目标分支推送、Pull Request 和手动触发接入 Windows Automated 交付门禁，并上传日志和 release bundle 证据。`n- 新增 `.gitattributes` 并修正验证器的 CRLF/LF 读取方式，消除 Windows 检出导致的伪失败；冻结合同和迁移登记哈希保持不变。`n- 更新根 `README.md` 和 R01 阶段索引，记录实际通过的 Windows 门禁和剩余最终验收项。'
)
$replacementTail = @'
## 11. GitHub Actions 自动验证收口

| 验证命令或操作 | 环境 | 结果 |
|---|---|---|
| R1 架构契约 smoke check | GitHub Actions `windows-latest` | 通过；合同 ID、引用、数量和状态 ID 唯一性均满足 |
| `npm ci`、前端契约、类型、截图与 Vite 生产构建 | GitHub Actions Windows | 通过 |
| Rust 格式、Clippy `-D warnings` 与 workspace tests | GitHub Actions Windows | 通过 |
| Tauri Windows release 构建 | GitHub Actions Windows | 通过 |
| release 客户端启动与运行日志扫描 | GitHub Actions Windows | 通过；应用版本 `0.23.0`，7 条记录，0 条无效记录，0 个运行时错误 |
| workflow run `30989439570` / job `92251837163` | 提交 `fc02ad51d01229cb2ea62fc20f623910ba49de7f` | `success` |
| artifact `8924033934` | GitHub Actions | `14115361` 字节；SHA-256 `85551aacdd43ba1e3516025ae510aefaaa8e11d61f433a701eaa884e292a47a1`；保留至 2026-08-19 |

## 12. 未执行验证与剩余风险

| 未执行项 | 原因 | 已完成替代验证 | 尚未排除的风险 |
|---|---|---|---|
| 真实 PostgreSQL 集成测试 | 按 R00 与用户当前交付规则保留到最终统一验收 | 静态数据库基线、workspace tests、Tauri 无数据库启动均通过 | 真实迁移、事务和数据库数据链仍需最终验收 |
| Windows Full 交互验收 | 当前门禁使用 `Automated` profile | release 构建、启动、bootstrap 和运行日志扫描通过 | 完整人工交互路径仍需最终验收 |
| 用户本机 Windows 实机验收 | GitHub hosted runner 不能替代用户设备环境 | `windows-latest` release 与启动验证通过 | 用户设备驱动、权限、显示和本地环境差异仍需最终验收 |

## 13. 当前限制与任务门禁

- R1-01 状态为 `DONE`；R1-02 开放为 `READY`。
- R1-03 至 R1-05 继续 `BLOCKED`，不得跨节点实施。
- 没有新增生产依赖、修改锁文件、数据库迁移、公共命令、模型保护资产或用户可观察行为。
- 当前环境看不到用户设备上的未提交或未跟踪文件；远端 `new-B` 不覆盖用户本地工作区。

## 14. 回退说明

- 回退基线：`36d34ba1ff73cbec575cf58594aa8c0329669496`。
- 架构契约可通过删除两份 JSON 和本记录回退。
- CI 可通过恢复旧 `.github/workflows/ci.yml` 并删除 `.gitattributes` 回退；验证器跨平台修复应与 Windows 门禁一起回退，避免恢复伪失败。
- 不需要数据库、迁移、依赖、公共接口或模型资产回滚。

## 15. 下一步

当前唯一可执行 Atomic Task：`R1-02 边界验证脚本`。
'@
$pattern = '(?s)## 11\. 未执行验证与原因.*\z'
if (-not [regex]::IsMatch($task, $pattern)) {
  throw "R1-01 trailing verification sections were not found."
}
$task = [regex]::Replace($task, $pattern, $replacementTail.TrimEnd() + "`n")
Write-RepoText $taskPath $task

Remove-Item ".github/workflows/r1-doc-finalize.yml" -Force
Remove-Item "scripts/r1-finalize-docs.ps1" -Force

$changed = @(git diff --name-only)
$expected = @(
  ".github/workflows/r1-doc-finalize.yml",
  "README.md",
  "docs/modular-rewrite/R01-architecture-composition/R01-01-模块边界契约.md",
  "docs/modular-rewrite/R01-architecture-composition/README.md",
  "scripts/r1-finalize-docs.ps1"
)
if ($changed.Count -ne $expected.Count -or @($changed | Where-Object { $_ -notin $expected }).Count -ne 0) {
  throw "Unexpected documentation scope: $($changed -join ', ')"
}

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add --all -- $expected
git commit -m "docs(r1): close R1-01 after Windows gate"
git push origin HEAD:new-B
