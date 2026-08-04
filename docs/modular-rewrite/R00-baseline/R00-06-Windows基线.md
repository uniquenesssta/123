# R0-06 Windows 基线：实施与验证记录

## 1. 基本信息

- 所属阶段：R00 基线冻结与可重复验收
- 任务状态：DONE
- 起始基线提交：`06732375bf3c3f40d94b5fdf2ff7609e07698f5a`
- 主要 Windows 证据测试提交：`2001154dff611e6326f2182d8e4a0b8aa35ca98b`
- 原始项目基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- 实施分支：`new-A`
- 完成日期：2026-08-04
- Draft PR：`#1`，仅用于触发验证，已关闭且未合并
- 主要 Windows workflow run：`30912862564`
- 主要 Windows job：`92003616257`
- 证据 artifact：`8894874465`
- artifact SHA-256：`9aacf759cd33bf4c01676465fbefe6bbe657fd7fae037e25a9429d162ea92e76`
- 对应任务书：`docs/football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md`

## 2. 原始问题与本节点目标

- 在全新 Windows 环境执行本次独立的 Automated 验收基线。
- 条件允许时补充 Full 模式；Full 依赖专用 PostgreSQL 和人工 GUI 操作。
- 保存本次新生成的 Windows acceptance log、runtime log 和 acceptance report，不复用历史日志。
- 记录真实通过项、失败项、环境阻塞、警告和未执行项。
- 不修改业务源码、公共接口、依赖、数据库迁移、模型实现或用户可观察行为。

## 3. 实际变更摘要

- 核对 `启动平台.bat`、`验证平台.bat`、`验收平台.bat`、`scripts/windows-acceptance.ps1`、Windows 验收契约及日志分析器。
- 使用临时 Windows GitHub Actions workflow 运行精确 Automated，并保留其失败。
- 在不改动生产源码的前提下，通过 runner 临时适配独立完成 TypeScript、Vite、锁定依赖、Tauri release 构建和 RuntimeOnly 启动验收。
- 生成并保存本次新的 runtime JSONL、startup acceptance report 和两份验收过程日志。
- 删除临时 workflow，关闭 Draft PR #1，未合并 `main`。
- Full 模式按用户要求推迟：数据库真实执行最后统一验证，且 GitHub runner 不具备人工 GUI 业务操作条件。

## 4. 新增文件

| 文件路径 | 唯一职责 | 上游 | 下游 | 新增原因 |
|---|---|---|---|---|
| `docs/modular-rewrite/R00-baseline/R00-06-Windows基线.md` | 保存 R0-06 的真实 Windows 结果、证据索引、限制和回退事实 | 任务书、Windows runner、验收契约 | R00 阶段出口判断 | 节点记录硬门禁 |

临时 `.github/workflows/r00-windows-baseline.yml` 只用于取得本节点证据，完成后已删除，不属于最终交付文件。

## 5. 修改文件

| 文件路径 | 修改前职责 | 本次修改 | 修改后职责 | 修改原因 | 影响范围 |
|---|---|---|---|---|---|
| `docs/modular-rewrite/R00-baseline/README.md` | R00 状态与阶段门禁索引 | 将 R0-06 标记为 DONE，记录 Windows 结果与阶段阻塞项 | 原职责不变 | 同步节点和阶段事实 | 文档 |
| `README.md` | 项目级验证与重写摘要 | 记录 R0-06 精确失败、适配启动通过及未执行项 | 原职责不变 | 根 README 同步门禁 | 文档 |

## 6. 移动或重命名文件

无。

## 7. 删除文件

- `.github/workflows/r00-windows-baseline.yml`：临时证据采集 workflow，完成后删除。
- 没有删除生产源码、测试、配置、依赖、迁移、模型或用户数据。

## 8. 模块、接口与数据流变化

- 最终分支仅保留文档变化，不新增生产运行链。
- 171 个公共 Tauri 命令、前端 DTO、Rust command 注册和错误语义保持不变。
- 46 个数据库迁移、数据库接口及不可变约束保持不变。
- 18 个公开模型保护文件和私有模型缺席边界保持不变。
- `package.json`、`package-lock.json`、Cargo manifests、`Cargo.lock`、Tauri 配置和正式 CI workflow 均未修改。
- runner 中对 `tauri.conf.json` 的 `beforeBuildCommand` 临时置空在构建后恢复，没有提交到仓库；其目的只是避免重复进入已单独执行过的 Windows 前端包装器。

## 9. 保持不变的行为

- UI、路由、页面状态、配置默认值、错误码、日志等级和用户可观察行为不变。
- 本节点未修复、删除、跳过或弱化精确 Automated 的失败。
- 独立启动验证仍使用仓库现有 release 应用、运行日志实现、startup profile 和验收分析器。
- 没有把 runner 临时适配结果描述为精确 Automated 通过。

## 10. Windows 执行环境

- Runner：GitHub-hosted Windows Server 2025。
- Windows：`Microsoft Windows NT 10.0.26100.0`。
- Node：`v22.23.1`。
- npm：`10.9.8`。
- Rust：`rustc 1.88.0`。
- Cargo：`1.88.0`。
- Git checkout：`core.autocrlf=false`、`core.eol=lf`。
- Cargo.lock 实际 SHA-256：`c42673480deb1da5ad581097a87521cf03347bda3d4ec49ee5aacdc4ad6bc449`，与发布契约一致。

## 11. 验证记录

| 验证命令或操作 | 结果 | 真实结论 | 证据 |
|---|---|---|---|
| Windows 验收体系静态契约 | 通过 | 8 个自动阶段、10 组运行链和三类合成日志反向门禁完整 | `r00-windows-contract-verification.txt` |
| LF 保真检出与 Cargo.lock 指纹 | 通过 | 默认 Windows CRLF 转换会破坏文本/字节型门禁；关闭自动转换后指纹与契约一致 | `r00-windows-environment.txt` |
| 精确 `windows-acceptance.ps1 -Mode Automated` | 失败 | 依赖同步阶段报告通过，但 `verify-searchable-select-diagnostics.mjs` 随后无法解析 `typescript` 包，Automated 在前端阶段退出 1 | `windows-acceptance-20260804-131523.txt`、`r00-windows-automated-console.txt` |
| Automated 失败前静态专项检查 | 通过 | 公开模型边界、命令契约、Cargo.lock、页面专项门禁等已执行通过 | Automated 控制台日志 |
| 17 个 UI 截图视口 | 通过 | 所有截图差异均在现有门禁阈值内 | Automated 控制台日志 |
| runner 中 `npm run setup` 补充依赖同步 | 通过并有警告 | 安装 19 个包；npm audit 有 1 个 moderate vulnerability | `r00-windows-release-workaround.txt` |
| 直接 TypeScript `--noEmit` | 通过 | 使用 TypeScript 的真实 Node 入口执行，未使用 Windows `.cmd` 包装器 | release workaround 日志后续步骤继续执行 |
| 直接 Vite production build | 通过并有警告 | 转换 50 个模块；存在单个产物大于 500 kB 的 Vite 警告 | `r00-windows-release-workaround.txt` |
| Cargo.lock 完整性与语义同步 | 通过 | 11 个本地包版本正确；`--locked` 无需更新 | release workaround 日志 |
| Tauri Windows release build | 通过并有警告 | release profile 14 分 25 秒完成；生成 EXE、MSI、NSIS | `r00-windows-release-workaround.txt` |
| Rust 编译警告 | 警告 | `LifecycleSplit.training_end` 未读取；`lifecycle_window_json` 未使用 | release workaround 日志 |
| `windows-acceptance.ps1 -Mode RuntimeOnly` | 通过 | release 客户端启动并创建本次独立运行日志 | `windows-acceptance-20260804-134651.txt` |
| startup acceptance report | 通过 | `profile=startup`、`status=pass`、7 条记录、3 个完成操作、0 个无效行、0 个运行时错误 | `windows-acceptance-20260804-134651.json` |
| 运行日志操作配对 | 通过 | `read_workspace_state`、`save_workspace_state`、`bootstrap` 均完成；启动事件存在 | `football-runtime-20260804T134700.829Z-pid7912-33300811.jsonl` |
| 独立证据上传 | 通过 | artifact `8894874465`，SHA-256 已记录，保留至 2026-09-03 | GitHub Actions run `30912862564` |
| Draft PR 清理 | 通过 | PR #1 已关闭，`merged=false` | GitHub PR 状态 |
| 临时 workflow 清理 | 通过 | 最终分支不保留 R0-06 临时 workflow | 删除提交 `9699cdebd5947be69ada6d6c119318a1f56ca6e5` |

## 12. 本次独立 runtime 证据

- Runtime log：`football-runtime-20260804T134700.829Z-pid7912-33300811.jsonl`。
- Session ID：`33300811-c3f4-49c5-96fe-88ce9a4cf3d3`。
- App version：`0.23.0`。
- 记录数：7。
- 无效行：0。
- 运行时错误：0。
- 完成操作：`bootstrap`、`read_workspace_state`、`save_workspace_state`。
- Startup report：`windows-acceptance-20260804-134651.json`。
- Report schema：`football.windows-acceptance-report.v1`。
- Contract：`football.windows-end-to-end-acceptance.v1` / `1.0.0`。
- `application_bootstrap` 必选覆盖组：通过。

## 13. 诊断运行与现有 Windows 问题

以下结果用于定位基线环境，不替代主要证据运行：

1. 临时 workflow 首轮因内联中文 checkout 路径被 Windows PowerShell 错误解码而未进入产品验证；后改为 ASCII checkout 加 Unicode 目录联接。该问题属于临时 workflow 编排，不归类为产品失败。
2. 默认 Windows CRLF checkout 会使固定 `\n` 文本断言和 Cargo.lock 原始字节哈希失败。LF 保真检出后两项均通过。
3. 诊断 run `30912298637` 在依赖已安装时推进到 TypeScript 阶段，但 `verify-frontend.mjs` 直接 `spawnSync(...\\tsc.cmd)` 返回 `EINVAL`。这说明 Windows 前端包装器还存在独立的 `.cmd` 子进程调用兼容问题。
4. 主要 run 的精确 Automated 又暴露依赖同步阶段与后续 ESM `typescript` 解析结果不一致；阶段显示通过，但后续脚本报告 `ERR_MODULE_NOT_FOUND`。
5. `验收平台.bat` 传入 `-LogDirectory`，但 `scripts/windows-acceptance.ps1` 未声明该参数，根验收入口当前无法按写法直接执行。
6. `.cargo/config.toml` 将目标目录设为源码父目录的 `.cargo-target`，但 `Find-ReleaseExecutable` 从源码目录下 `.cargo-target` 查找。本次仅在 runner 建立目录联接，不修改正式脚本。

这些问题均未在 R0-06 中夹带修复，必须作为独立修复任务处理并重新运行精确 Automated。

## 14. 未执行验证与原因

| 未执行项 | 原因 | 已完成替代验证 | 尚未排除风险 | 后续操作 |
|---|---|---|---|---|
| Windows Full 模式 | 需要专用 PostgreSQL 和人工 GUI 业务操作；用户要求数据库最后统一验证 | 精确 Automated 基线、完整 release 构建、RuntimeOnly startup | 数据库纵向链、10 组人工操作、取消、数据库断连等 Full 覆盖未确认 | 最终统一验证执行 Full |
| PostgreSQL 迁移幂等及 18 个 ignored tests | 用户明确延期 | R0-04 静态迁移和测试契约冻结 | 数据库运行风险仍存在 | 最终统一验证执行 |
| 用户本机 Windows 10/11 | 当前无法访问用户设备 | GitHub-hosted Windows Server 2025 | WebView2、权限、杀毒、路径、显卡和本机策略差异 | 最终实机复核 |
| 精确 Automated 全通过 | Windows 依赖同步和 `.cmd` 启动器问题阻塞 | 底层 TypeScript/Vite、release 和 RuntimeOnly 分开验证 | 原始一键入口仍不可重复通过 | 独立修复后重跑 |
| R0-05 Clippy/workspace tests | `cargo fmt --check` 失败导致跳过 | 本次 release 编译成功 | lint 和单元/集成回归仍未排除 | 格式修复后执行完整 `npm run verify:rust` |

## 15. 关键决策与偏差

- 采用 GitHub-hosted Windows Server 2025 获取真实 Windows 结果，不用 Linux 推断 Windows。
- 精确 Automated 失败原样保留，不修改门禁或删除检查。
- 为满足任务书的本次独立 runtime log/report 要求，在 runner 中先显式安装依赖并直接调用 TypeScript/Vite Node 入口，再临时跳过重复的 `beforeBuildCommand` 包装器。
- runner 临时 Tauri 配置修改在构建后恢复；目标目录联接只存在于一次性 runner。
- release 构建使用未改动的生产 Rust/Tauri 源码和锁定依赖，生成实际 EXE、MSI 和 NSIS。
- Full 模式没有伪造，也没有使用历史日志代替。

## 16. 回退说明

- 回退基线：`06732375bf3c3f40d94b5fdf2ff7609e07698f5a`。
- 删除本节点记录并恢复阶段 README、根 README 的 R0-06 前状态。
- Draft PR 已关闭且未合并；临时 workflow 已删除。
- 不涉及业务数据、数据库、生产源码、依赖或运行配置回退。

## 17. 完成结论

- 是否获得真实 Windows 基线：是。
- 精确 Automated 是否通过：否。
- 是否保存本次独立 runtime log：是。
- 是否保存本次独立 startup acceptance report：是，状态为 PASS。
- Tauri Windows release 是否构建成功：是，使用 runner 临时验证包装器适配。
- Full 模式是否执行：否，数据库和人工 GUI 按明确边界延期。
- 是否修改业务源码或公共接口：否。
- R0-06 的“记录真实 Windows 基线并取得独立启动证据”目标是否完成：是。
- R00 阶段出口门禁是否通过：否；精确 Automated、Rust 完整验证、数据库实跑和 Full 仍有明确缺口。
- 是否允许 `R1-01` 进入 READY：否。
