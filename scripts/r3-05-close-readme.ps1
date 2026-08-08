$ErrorActionPreference = "Stop"

$path = "README.md"
$source = Get-Content -Raw -Encoding UTF8 $path

$oldR3 = @'
- R3 已从 R2 完成提交 `7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f` 建立独立分支 `new-C`。R3-01 Application Ports、R3-02 Database Service、R3-03 Competition / Rules Services 均已完成。R3-04 已将 35 个球队/球员/教练/实体引用职责拆入 Teams / Players Services；用户随后提供 clean 工作区、rustfmt、R3-04 专项、architecture、Application check 与 33/33 tests 的本机通过结果并明确授权进入 R3-05，但未提供完整 frontend / Rust 与 runtime 烟测，因此 R3-04 仍为 `VERIFYING`。R3-05 已删除旧 `crates/application/src/player_catalog.rs`，将剩余 19 个阵型/比赛/阵容/阵容预设职责迁入 `services/lineups/` 与 19 个对应 Use Cases，并以 `FormationPort`、`MatchCatalogPort`、`LineupPort`、`LineupPresetPort` 4 个既有 Ports 保持公共 Application/Tauri 契约；`read_match_exchange` 仅提升 workspace 可见性以满足既有 MatchCatalogPort，SQL、参数、返回结构与数据库行为不变。clean 提交 `7e3fddeafcd32cc45e293fa9a7aeb05c7c66d4ec` 的 Public Platform CI run `31260698438` / job `93110942400` 已通过 architecture 与完整 Windows Automated，包括 frontend、17 个截图回归视口、TypeScript、Vite、完整 Rust/Clippy/workspace tests、Tauri release 构建和 release 启动日志扫描；artifact `9022970030`，SHA-256 `275e17a78db9d5205d49401a1a1d20ed91f08102594d2d04c339051165beb052`。R3-05 当前为 `VERIFYING`，等待用户 Windows 本机复核与非破坏性 runtime 烟测；R3-06 继续 `BLOCKED`。
'@
$newR3 = @'
- R3 已从 R2 完成提交 `7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f` 建立独立分支 `new-C`。R3-01 Application Ports、R3-02 Database Service、R3-03 Competition / Rules Services 均已完成。R3-04 已将 35 个球队/球员/教练/实体引用职责拆入 Teams / Players Services，历史状态继续独立保留为 `VERIFYING`。R3-05 已删除旧 `crates/application/src/player_catalog.rs`，将剩余 19 个阵型/比赛/阵容/阵容预设职责迁入 `services/lineups/` 与 19 个对应 Use Cases，并以 `FormationPort`、`MatchCatalogPort`、`LineupPort`、`LineupPresetPort` 4 个既有 Ports 保持公共 Application/Tauri 契约；`read_match_exchange` 仅提升 workspace 可见性，SQL、参数、返回结构与数据库行为不变。clean 实施提交 `7e3fddeafcd32cc45e293fa9a7aeb05c7c66d4ec` 的 Public Platform CI run `31260698438` / job `93110942400` 已通过完整 Windows Automated；用户随后在最终分支完成 clean 工作区、rustfmt、Lineups 专项、architecture、Application 33/33、完整 frontend、完整 Rust/Clippy/workspace tests 与 `tauri:dev`。本机 runtime JSONL 共 280 条，除 3 条预期输入校验与 3 条公开模型运行时未分发的既有错误外，无 Lineups、SQL、migration、panic 或连接失败；预设保存、应用预检、双方阵容原子创建与 `ready_for_model=true` 阵容链均实际跑通。R3-05 状态为 `DONE`，R3-06 Prediction Service 已开放为 `READY`。
'@

if (-not $source.Contains($oldR3)) { throw "README R3 status paragraph not found" }
$source = $source.Replace($oldR3, $newR3)

$oldStatus = '已创建 `R00-stage-completion.md`、`R01-stage-completion.md` 与 `R02-stage-completion.md`。R1、R2 阶段均已关闭；R3-01 Application Ports、R3-02 Database Service、R3-03 Competition / Rules Services 状态均为 `DONE`，R3-04 Teams / Players Services 与 R3-05 Lineups Service 均为 `VERIFYING`，R3-06 Prediction Service 为 `BLOCKED`。详细状态见 `docs/modular-rewrite/R03-application-services/README.md`。'
$newStatus = '已创建 `R00-stage-completion.md`、`R01-stage-completion.md` 与 `R02-stage-completion.md`。R1、R2 阶段均已关闭；R3-01 Application Ports、R3-02 Database Service、R3-03 Competition / Rules Services 与 R3-05 Lineups Service 状态为 `DONE`，R3-04 Teams / Players Services 的历史状态仍为 `VERIFYING`，R3-06 Prediction Service 为 `READY`。详细状态见 `docs/modular-rewrite/R03-application-services/README.md`。'
if (-not $source.Contains($oldStatus)) { throw "README R3 summary line not found" }
$source = $source.Replace($oldStatus, $newStatus)
Set-Content -Path $path -Value $source -Encoding UTF8 -NoNewline

git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git rm .github/workflows/r3-05-close-readme.yml scripts/r3-05-close-readme.ps1
git add README.md
git diff --check
git commit -m "docs(r3): close lineups service in root README"
git push origin HEAD:rewrite/r3-05-lineups-service
