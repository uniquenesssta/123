# R03-03 Competition / Rules Services

## 状态

`VERIFYING`

Competition / Rules Services 源码重写、实施侧验证、用户 Windows 本机最小验证、完整 frontend / Rust 回归与非破坏性 runtime bootstrap 烟测均已完成。当前仅等待用户拉取最后一处保护资产验证器确定性修复后，重新执行直接 `verify_protected_assets.mjs`；该单项通过后即可关闭 R3-03，R3-04 在此之前继续保持 `BLOCKED`。

## 基线与范围

- 分支：`new-C`，实施保护分支：`rewrite/r3-03-competition-rules`
- R3-03 起点：`c4ee3b609cf950273a50e9a325023f9e082d6aba`（R3-02 `DONE` 后开放 R3-03）
- 首个完整重写提交：`889a543416979d9461600c24ae84f140551727aa`
- 验证器兼容修复：`4c0f63c35b874b2f02a7dd6398abfd97f55a902b`
- 保护资产验证器确定性修复最终提交：`36e80f7c6209e4b0a8f3e7854147dcbd919df332`
- 任务范围：赛事层级、规则包、赛事绑定及 bootstrap / 数据库初始化中的相关 Application 编排。
- 排除范围：具体 PostgreSQL SQL、迁移、Prediction 路由预览/模型调用、Teams/Players、Tauri DTO、前端状态、模型实现。

## 实际实施

### 1. Competition Service / Use Cases

新增 `crates/application/src/services/competition/` 与 `crates/application/src/use_cases/competition/`，按职责拆分：

- 创建赛事；
- 删除赛事；
- 创建赛季；
- 创建阶段；
- 创建轮次；
- 读取赛事层级目录；
- 赛事/阶段/轮次代码生成辅助职责。

原 `crates/application/src/competition.rs` 已删除。既有赛事代码自动生成、日期区间、状态、赛事类型与中文错误语义保持不变；ApplicationService 同名公开方法只负责取得活动数据库会话并委托 CompetitionService。

### 2. Rules Service / Use Cases

新增 `crates/application/src/services/rules/` 与 `crates/application/src/use_cases/rules/`，按职责拆分：

- 规则包结构与参数身份校验；
- 规则包工厂与公开默认模板；
- 注册用户规则包；
- 注册内置规则包；
- 创建赛事规则绑定；
- 读取规则包与绑定目录。

原 `crates/application/src/rule_packages.rs` 已删除。模型存在性、赛事类型支持、模型参数校验、规则包参数身份校验、类型默认绑定优先级与既有标签语义全部保留；`default_rule_package_template` 继续从 Application crate 根公共导出。

### 3. Ports / PostgreSQL 边界

`composition/port_registry.rs::ActiveDatabase` 实现：

- `CompetitionHierarchyPort`
- `RulePackagePort`
- `RuleRoutingPort`

Service / Use Case 不直接导入 `football_persistence_postgres`、`PostgresStore`、SQLx、PgPool 或 SQL Row；具体 PostgreSQL 调用继续只存在于组合根适配器，未修改持久化 SQL 或迁移。

R3-03 只把规则包注册、赛事绑定和目录读取切入 Rules Service；Prediction 的 route preview / readiness / model invocation 仍留给 R3-06，不提前迁移。

### 4. Database 初始化与 Bootstrap

数据库连接后的内置规则包注册已从 Database facade 的具体持久化流程切换为：

`Database facade -> RulesService -> register_built_ins use case -> RulePackagePort / RuleRoutingPort`

P4 persistence artifacts 与 research artifacts 的既有初始化顺序继续保留。

Bootstrap 的赛事、赛季、阶段、轮次读取改为 CompetitionService；规则包与赛事绑定读取改为 RulesService。最近模型运行等未属于 R3-03 的路径保持现状。

### 5. Tauri / 公共兼容

`src-tauri/src/commands/competition.rs` 的 7 个既有公共命令名称、参数、返回 DTO 和委托方法保持不变：赛事创建/删除、赛季、阶段、轮次、规则包注册、赛事绑定创建。

未修改公共 ApplicationService 方法名、Serde 数据结构、数据库 Schema、配置、生产依赖、Cargo.lock 或模型实现与私有资产范围。

## 专项门禁

新增 `scripts/verify-competition-rules-service.mjs` 并接入：

- `npm run verify:competition-rules-service`
- `npm run verify:architecture`
- `npm run verify:frontend`

门禁锁定目标目录、旧根文件删除、ApplicationService/组合根服务聚合、3 个 Ports 适配、bootstrap/初始化委托边界、Service/Use Case 无具体 PostgreSQL 泄漏、Tauri 7 条公共调用链以及验证入口。

R3-03 同步更新了 R3-02 Database Service 验证器：旧验证器原先硬编码要求已被 R3-03 正确替代的 `register_built_in_rule_packages` 字面量，现改为验证 `RulesService::register_built_ins` 新边界并明确拒绝旧实现；没有删除 R3-02 的生命周期、reset、安全确认或 PostgreSQL 隔离门禁。

公开模型边界验证器原先仍读取已删除的 `crates/application/src/rule_packages.rs`，现已跟随唯一职责 owner 改读 `use_cases/rules/package_factory/mod.rs`。随后用户直接执行 `verify_protected_assets.mjs` 暴露其聚合排序依赖宿主 locale 的历史缺陷；已把基础验证器自身改为 ordinal path sort，使直接入口与 deterministic wrapper 使用同一确定性语义，并同步刷新该验证器自身的受保护指纹与派生聚合值。没有修改模型源码、参数、Profile、私有资产范围或放宽保护门禁。

## 实施侧验证

Windows 2025 严格验证 run `31248365735` / job `93080599447` 在实施保护分支上全部通过，并生成最终同步提交 `fc93103fb4327bbafea7b800984a971f5bf1f328`。临时 workflow 已从最终树删除。

已通过：

- `cargo fmt --all -- --check`
- Domain 类型清单重新生成并验证：365 个类型、365 个公共兼容类型、299 个 PostgreSQL 映射类型；当前扫描 129 个来源文件。
- `node scripts/verify-competition-rules-service.mjs`：21 个 Service / Use Case Rust 文件通过职责与调用链检查。
- `npm run verify:architecture`：模块边界、状态所有权、受保护导入、Domain 根出口、Application Ports、Database Service 与 Competition/Rules Service 全部通过。
- `node scripts/verify-protected-assets-deterministic.mjs`：模型保护边界指纹通过。
- `cargo check --locked -p football-application`
- `cargo test --locked -p football-application`
- `git diff --check`

保护资产确定性修复 workflow run `31249193592` 进一步验证：直接 `verify_protected_assets.mjs` 与 deterministic wrapper 均在同一代码树通过，临时 workflow 已自删除。

## 用户 Windows 本机验证

用户本机已通过：

- 工作区干净、`cargo fmt --all -- --check`；
- `npm run verify:competition-rules-service`；
- `npm run verify:architecture`；
- `cargo check --locked -p football-application`；
- `cargo test --locked -p football-application`：31/31；
- 完整 `npm run verify:frontend`：全部静态门禁、17 个截图视口、TypeScript 与 Vite production build 通过；Vite 仅保留既有大 chunk warning；
- 完整 `npm run verify:rust`：workspace Clippy `-D warnings` 与 workspace tests 无失败；18 个真实 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 按既有设计保持 `ignored`；
- `npm run tauri:dev` 正常编译并启动原数据库。

本次 runtime JSONL 共 98 条：97 条 `info`，1 条前端用户动作 `error` 为阵容页在未选择球员时点击添加而返回“请先选择球员”，属于既有输入校验，不是数据库、Competition/Rules Service、panic 或基础设施失败。`bootstrap` 在 450 ms 内完成且 `connection_error=null`；当前 bootstrap 实现只有在 CompetitionService `load_hierarchy` 与 RulesService `load_catalog` 均成功后才会返回，因此该运行已经覆盖原数据库赛事层级、规则包与赛事绑定读取边界。

用户直接执行旧版本 `node scripts/verify_protected_assets.mjs` 时唯一失败为聚合排序的 locale 差异；完整 frontend 中 deterministic 保护门禁同时通过。该根因已经在远端修复，现只需用户拉取后重跑直接入口确认本机同样通过，不需要重复 frontend、Rust 或 runtime。

## 尚未完成

仅剩：

- `git pull`
- `node scripts/verify_protected_assets.mjs`

该直接入口本机通过后即可把 R3-03 标记为 `DONE` 并开放 R3-04。

真实 PostgreSQL destructive/reset 集成不使用用户原数据库；18 个需要 `FOOTBALL_TEST_DATABASE_URL` 的专用 PostgreSQL 集成测试若未配置测试库，继续按既有设计保持 `ignored`，不计为已执行。

## 回退与下一步

R3-03 可回退到起点 `c4ee3b609cf950273a50e9a325023f9e082d6aba`。不得恢复已删除的职责混合根文件或绕过 Services / Use Cases / Ports 重新直接访问 PostgreSQL。

状态保持 `VERIFYING`。待用户本机直接保护资产验证器复跑通过后关闭 R3-03，并开放 R3-04 Teams / Players Services。
