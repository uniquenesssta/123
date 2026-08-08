# R03-02 Database Service

## 状态

`DONE`

Database Service 已完成源码切换、实施侧专项验证、Windows 本机最小验证、完整阶段回归与运行时烟测。R3-02 正式关闭，R3-03 Competition / Rules Services 开放为 `READY`。

## 基线与范围

- 分支：`new-C`
- R3-02 起点：`7194a46b520c23aaeedef3aa785b97a5090d8c78`（R3-01 `DONE` 后开放 R3-02）
- 主要实现提交：`cd754f79456b96b3e66ac45b119f61609346e06d`
- 验证器兼容修复提交：`3530cd774a995bfae5ca2d5621279fd74ac34289`
- 任务范围：数据库连接、迁移、恢复、health、statistics、reset，以及 ApplicationService 兼容委托。
- 排除范围：具体 PostgreSQL SQL、Tauri DTO 结构、前端状态、模型实现。

## 实际实施

### 1. Database Service 与状态所有权

新增 `crates/application/src/services/database/`：

- `service.rs`：唯一持有活动数据库会话 `DatabaseService.session`，负责 prepare / activate / disconnect / reset 生命周期协调。
- `facade.rs`：保留既有 `ApplicationService` 数据库公开方法，只委托 Database Service，并负责连接成功后的既有初始化与 worker 启动。
- `bootstrap.rs`：单独承担 bootstrap 数据聚合，health/statistics 通过用例调用，其余既有目录读取继续使用当前持久化 store。
- `mod.rs`：只声明子模块并显式导出 `DatabaseService`。

`ApplicationService` 不再直接持有 `RwLock<Option<ActiveDatabase>>`；活动数据库状态唯一所有者切换为 `crates/application/src/services/database/service.rs::DatabaseService.session`。

### 2. Database Use Cases

新增 `crates/application/src/use_cases/database/`：

- `connect/`：按既有顺序执行 migrate，再执行 interrupted work recovery；带 fake port 单元测试锁定顺序。
- `migrate/`：只调用 `DatabaseLifecyclePort::migrate`。
- `health/`：只读取 `DatabaseObservabilityPort::health`。
- `statistics/`：只读取 `DatabaseObservabilityPort::statistics`。
- `reset/`：保留数据库名称强确认、目标数据库一致性门禁、彻底重建失败后的恢复迁移尝试和既有中文错误语义；带 fake port 单元测试。

### 3. PostgreSQL 边界

具体 `PostgresStore` 连接、迁移、恢复、reset、health、stats 适配仍集中在 `crates/application/src/composition/port_registry.rs`。Service 与 Use Case 不直接导入 SQLx、PgPool、PostgresStore 或 SQL Row。

`ActiveDatabase` 在组合根适配器中实现 `DatabaseLifecyclePort` 与 `DatabaseObservabilityPort`，持久化错误继续映射为 Application Port 错误。

### 4. Tauri reset 调用链

`src-tauri/src/commands/database.rs` 不再直接连接 PostgreSQL 或调用 `reset_to_pristine`。清空流程为：

`Tauri command -> ApplicationService -> DatabaseService -> reset use case -> DatabaseLifecyclePort -> PostgreSQL adapter`

Tauri 保留既有配置读取、运行日志、P4 worker 恢复和返回 DTO 语义。

### 5. 旧实现清理

删除 `crates/application/src/database.rs`。数据库职责不再由根级混合文件承担，唯一入口已切换到 `services/database/` 与 `use_cases/database/`。

## 实施期问题与修复

第一轮实施验证暴露两个真实兼容问题：

1. `active_store` 随文件迁入嵌套模块后仍使用 `pub(super)`，导致 crate 内既有调用方出现 `E0624`；已改为 `pub(crate)`，恢复原有 crate 内可见范围，不扩大外部公共 API。
2. `recover_interrupted_api_workspace_operations()` 返回恢复数量，新 `recover_interrupted_work()` 误把 `Result<u64, _>` 作为 `Result<(), _>` 返回；已显式丢弃计数并恢复 `()` 语义。

同时删除两个已无调用的 crate-private 根别名，没有增加 lint 抑制或降低门禁。

完整 frontend 首轮又发现 `verify-database-reset.mjs` 使用连续字符串匹配 rustfmt 后的链式调用，误判 `preflight_database_reset` 缺失。验证器已改为忽略纯空白差异后检查同一调用链；业务强确认、二次数据库名称校验和 reset 委托均未放宽。

## 契约与行为

- `ApplicationService` 既有数据库公开方法名称与调用语义：保持。
- Bootstrap / DatabaseHealth / DatabaseStats 数据结构：保持。
- 数据库配置键与保存格式：保持。
- reset 数据库名称确认与错误提示语义：保持。
- PostgreSQL Schema、0001–0046 迁移 SQL、历史数据格式：未修改。
- Tauri 公共命令与 DTO：未修改结构。
- 模型 API、P4/P7 保护资产：未修改。
- 生产依赖与 `Cargo.lock`：未新增或升级。

## 验证记录

### 实施侧 Windows 专项

Temporary R3-02 Validate run `31244006019` / job `93069490517` 通过：

- `cargo fmt --all -- --check`
- `node scripts/verify-application-ports.mjs`
- `node scripts/verify-application-composition.mjs`
- `node scripts/verify-database-service.mjs`
- `node scripts/verify-database-reset.mjs`
- `node scripts/verify-domain-type-inventory.mjs`
- `node scripts/verify-protected-assets-deterministic.mjs`
- `npm run verify:architecture`
- `cargo check --locked -p football-application`
- `cargo test --locked -p football-application`

临时 workflow 已从最终树删除。

### 用户 Windows 本机验收

本机最小验证通过：

- 工作树在验证开始时为空。
- `cargo fmt --all -- --check`
- `npm run verify:database-service`
- `npm run verify:architecture`
- `cargo check --locked -p football-application`
- `cargo test --locked -p football-application`：30/30 通过。

阶段回归通过：

- `node scripts/verify-database-reset.mjs`
- `npm run verify:frontend`：Database Service、保护资产、171 Tauri 命令、17 个截图视口、TypeScript、Vite production build 全部通过；仅保留既有大 chunk warning。
- `npm run verify:rust`：Cargo.lock、rustfmt、workspace all-targets Clippy `-D warnings`、workspace tests 全部通过。
- workspace tests 中 PostgreSQL crate 74 个普通测试通过；18 个真实 PostgreSQL 集成测试因未设置 `FOOTBALL_TEST_DATABASE_URL` 按既有安全设计保持 `ignored`，未记为已执行。

运行时烟测通过：

- `npm run tauri:dev` 正常完成前端启动、desktop 编译并启动应用。
- 上传的本次 runtime JSONL 共 48 条记录，48 条均为 `info`，无 `warning` / `error` / `critical` / `panic`。
- `bootstrap` 完成，`connection_error = null`。
- 原数据库读取链实际完成：教练 26 条、阵型 17 条、球队列表与球队详情读取成功；阵容、规则页导航、Analytics 与 Postmatch 查询均完成，没有数据库或迁移错误。
- 本次运行日志实际绝对路径为 `F:\FOODBALL\logs\football-runtime-20260808T074530.382Z-pid33064-36b99d47.jsonl`。因此从源码目录 `F:\FOODBALL\123 r3` 执行 `Get-ChildItem .\logs` 会找不到目录；这是当前 runtime root discovery 的既有路径行为，不是 Database Service 失败，本节点未修改运行日志目录策略。

真实 destructive reset 未在用户原数据库执行。后续如需 destructive integration，只允许使用名称与环境均满足项目安全门禁的专用测试数据库。

## 回退

节点源码可回退到 R3-02 起点 `7194a46b520c23aaeedef3aa785b97a5090d8c78`。不得通过复制恢复已删除的 `database.rs`，也不得绕开 Ports/Service/Use Case 边界恢复直接 PostgreSQL 调用。

## 下一步

R3-02 已关闭为 `DONE`。R3-03 Competition / Rules Services 状态切换为 `READY`；开始 R3-03 前重新读取当前根规则、R3 任务书、Competition/Rules Ports、Application 调用面和对应 PostgreSQL 适配边界。