# R01-04 Tauri 组合根

## 1. 基本信息

- 所属阶段：R01 架构契约与空壳组合根
- 任务状态：`VERIFYING`
- 实施分支：`new-B`
- 开始基线：`8db5f460f25887edac6a6bf95932de6c46164e9a`
- 正式实现提交：`c73a6bd1d28435700274fef4fa115e8f97ce294e`
- 实施日期：2026-08-06
- 目标平台：Windows

## 2. 实际解决的问题

实施前，`src-tauri/src/lib.rs` 同时负责：

- Tauri Builder 构造；
- dialog 插件安装；
-配置目录解析与启动日志记录；
- `AppState` 构造和注入；
- 171 条 Tauri 命令注册；
- context 生成、运行和启动失败处理。

`AppState` 又定义在 `src-tauri/src/commands.rs`，导致命令出口同时承担共享状态所有权。该结构使入口文件持续吸收组合职责，也无法单独验证状态、命令注册和启动错误边界。

## 3. 完成后的职责边界

### `src-tauri/src/lib.rs`

只保留模块声明和公共 `run()`，并将执行委托给 `bootstrap::run()`。

### `src-tauri/src/bootstrap/mod.rs`

只声明并组合 bootstrap 子模块，显式导出 crate 内唯一 `AppState`。

### `src-tauri/src/bootstrap/application.rs`

唯一负责：

- 构造 Tauri Builder；
- 安装既有 dialog 插件；
- 调用状态安装；
- 调用命令注册；
- 使用既有 Tauri context 运行应用。

### `src-tauri/src/bootstrap/state.rs`

唯一负责：

- 定义 `AppState`；
- 解析既有应用配置目录；
- 创建并记录既有 runtime log session；
- 构造 ApplicationService、日志、OpenAI Profile、workspace 和 API request 状态；
- 通过 `app.manage` 注入状态。

### `src-tauri/src/bootstrap/command_registry.rs`

只登记现有 171 条命令，不实现命令；命令名称和注册顺序与冻结命令契约完全一致。

### `src-tauri/src/bootstrap/error.rs`

只负责启动阶段 I/O 错误映射和最终启动失败语义，继续使用原提示：`足球赛事模型平台启动失败`。

## 4. 入口与所有权切换

- 公共入口保持 `src-tauri/src/lib.rs::run` 不变。
- 实际组合入口切换到 `src-tauri/src/bootstrap/mod.rs::run`。
- Tauri Builder 所有者切换到 `bootstrap/application.rs`。
- `AppState` 所有者从 `commands.rs` 切换到 `bootstrap/state.rs`。
- 命令注册所有者从 `lib.rs` 切换到 `bootstrap/command_registry.rs`。
- `commands.rs` 只通过 crate 内公开出口引用 `AppState`，不再重复定义共享状态。

## 5. 公共契约与行为

保持不变：

- 171 条 Tauri 命令名称；
- 命令注册顺序；
- 命令参数和返回类型；
- 前端调用集合；
- dialog 插件安装；
- 配置目录和持久化文件路径；
- runtime log 启动记录字段；
- `ApplicationService::new()` 构造语义；
- API workspace request map 类型和生命周期；
- Tauri context、窗口行为和启动失败提示；
- DTO、Schema、数据库格式、迁移、配置键、错误码和日志等级。

未新增生产依赖，未修改 `Cargo.lock`。

## 6. 保护资产校正

R1-03 后的基线中，`crates/application/src/model_shell/mod.rs` 的排版与 Rust 1.88 `cargo fmt --check` 存在冲突，而该文件同时属于保护清单。

本任务只执行 Rust 1.88 标准排版，不改变：

- 模块声明；
- re-export 集合；
- 模型 ID；
- 模型行为；
- 保护目录和允许文件集合。

随后仅同步更新该文件的 Git blob SHA-1、派生 SHA-256 和清单聚合值：

- Git blob SHA-1：`93933d52039f1b910a64189f7c292610f47ce6f3`
- 文件指纹 SHA-256：`47c05a5e621def716e9922a044b34139d674b17302371eda9aa421782e94e199`
- 聚合 SHA-256：`a67bd371700ed3f9d4ed49a61e24338f9a022909ba369786e661d15336c09ef6`

为消除 Windows ICU/区域设置对 `localeCompare` 排序的影响，新增确定性代码点排序入口；该入口仍执行原保护验证器的全部单文件、目录集合、禁止路径和私有资产缺席检查。

## 7. 文件清单

以下清单以 R1-03 完成基线到本节点最终交付树的实际差异为准。

### 新增

- `src-tauri/src/bootstrap/mod.rs`
- `src-tauri/src/bootstrap/application.rs`
- `src-tauri/src/bootstrap/state.rs`
- `src-tauri/src/bootstrap/command_registry.rs`
- `src-tauri/src/bootstrap/error.rs`
- `scripts/verify-tauri-bootstrap.mjs`
- `scripts/verify-protected-assets-deterministic.mjs`
- `docs/modular-rewrite/R01-architecture-composition/R01-04-tauri-组合根.md`

### 修改

- `README.md`
- `architecture/command-contract.json`
- `architecture/module-boundaries.json`
- `architecture/protected-assets.json`
- `architecture/state-ownership.json`
- `crates/application/src/model_shell/mod.rs`
- `docs/modular-rewrite/R01-architecture-composition/README.md`
- `scripts/verify-command-contract.mjs`
- `scripts/verify-frontend.mjs`
- `scripts/verify-api-runtime-diagnostics.mjs`
- `scripts/verify-api-workspace.mjs`
- `scripts/verify-database-reset.mjs`
- `scripts/verify-entity-relationships.mjs`
- `scripts/verify-force-team-delete.mjs`
- `scripts/verify-formation-usage.mjs`
- `scripts/verify-history-scoreline-ui.mjs`
- `scripts/verify-match-lineup-chain.mjs`
- `scripts/verify-match-review-package.mjs`
- `scripts/verify-match-workflow-ui.mjs`
- `scripts/verify-monthly-workbooks.mjs`
- `scripts/verify-openai-profile-ui.mjs`
- `scripts/verify-parameter-lifecycle.mjs`
- `scripts/verify-postmatch-settlement.mjs`
- `scripts/verify-release-acceptance.mjs`
- `scripts/verify-stage-e2-lineup-presets.mjs`
- `scripts/verify-team-package.mjs`
- `scripts/verify-team-player-management.mjs`
- `scripts/verify-workspace-ui.mjs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/lib.rs`

### 移动或重命名

无。

### 删除

无基线文件被删除。临时实施脚本和临时 GitHub Actions 工作流只用于受控切换，未保留在最终交付树中。

## 8. 永久验证门禁

新增 `scripts/verify-tauri-bootstrap.mjs`，验证：

- bootstrap 目录必须且只能包含五个职责文件；
- `lib.rs` 必须是薄入口，不得重新出现 Builder、setup、manage 或命令注册；
- Builder、状态、命令注册和错误映射必须由指定模块拥有；
- `AppState` 字段和所有权契约一致；
- 171 条命令集合、顺序和冻结契约完全一致；
- `commands.rs` 不得重复定义 `AppState`；
- 启动日志、插件、context 和启动失败提示保持不变。

该专项门禁和确定性保护资产入口均已接入 `scripts/verify-frontend.mjs`。

- 同步迁移 19 个既有验证器的 Tauri 注册表或状态读取路径，防止旧 `lib.rs` 硬编码产生伪失败。

## 9. 已执行验证

专用 Windows workflow：

- workflow run：`31027424414`
- job：`92379334852`
- 结论：`success`
- Node：22.23.2
- Rust：1.88.0
- runner：Windows Server 2025

实际通过：

- 模块边界验证；
- 状态所有权验证；
- 受保护导入验证；
- 171 条命令权威契约验证；
- 171 条命令静态三方一致性验证；
- 18 个保护文件指纹、保护目录集合及私有 P4/P7 资产缺席验证；
- `cargo fmt --all -- --check`；
- `cargo check --locked -p football-match-model-desktop`；
- `git diff --check`。

## 10. 待执行验证与当前状态

### Windows release 启动观测修正

- 正式 Automated run `31029576871` 已通过前端、Rust 与 Tauri bundle 构建，但单次 release 启动 45 秒内未观测到 runtime log。
- 生命周期探针 run `31032667039` 已确认公共入口、Builder、setup、状态安装和日志写入十个节点全部到达，组合根本身无启动回归。
- A/B run `31034333416` 在相同 Windows runner 布局中验证打包前与打包后 EXE 均成功启动并建立 runtime log；两者 SHA-256 分别为 `f9f1afaaeb19ade39642be7d158be1cb17d078d14650871328e469c961bd344d` 与 `fd60ab2a04c93df558478554fa4dfe49522c686d19deb35728a4cdadb3882939`。
- 因此未修改 Tauri 组合根或打包流程；Windows 验收器改为按启动前日志路径集合识别新 session，首次 45 秒超时后最多重启一次，并为每次尝试保留 stdout/stderr。进程提前退出仍立即失败，连续两次超时仍硬失败。
- 该修正只影响外部启动观测与诊断，不改变产品入口、171 条命令、状态所有权、bundle 产物、依赖、迁移或模型保护资产。
- R1-04 保持 `VERIFYING`，等待清理临时工作流后的正式 Windows Automated。

清理临时实施文件后的最终 HEAD 仍需通过正式 `Public Platform CI`，重新覆盖：

- 完整架构门禁；
- `npm run verify:frontend`；
- Rust fmt、Clippy 和 workspace tests；
- Tauri Windows release 构建；
- release 客户端启动；
- runtime 日志扫描；
- 验证证据上传。

因此本节点保持 `VERIFYING`，R1-05 继续 `BLOCKED`。

真实 PostgreSQL、Windows Full 交互验收和用户本机 Windows 实机验收继续保留到最终统一验收。

## 11. 回退

回退本节点时应整体回退 R1-04 原子提交，不手工复制旧入口：

- 恢复 `src-tauri/src/lib.rs` 的组合职责；
- 恢复 `commands.rs` 中的 `AppState`；
- 恢复三份架构契约和命令注册路径；
- 恢复保护清单和纯排版文件；
- 删除本节点新增的 bootstrap 和专项验证文件。
