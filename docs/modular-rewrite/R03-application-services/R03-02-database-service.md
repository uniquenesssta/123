# R03-02 Database Service

## 状态

`VERIFYING`

Database Service 源码切换与实施侧专项验证已完成；节点仍等待 Windows 本机最小验证、阶段回归与运行时烟测，因此不得标记为 `DONE`，R3-03 继续保持 `BLOCKED`。

## 基线与范围

- 分支：`new-C`
- R3-02 起点：`7194a46b520c23aaeedef3aa785b97a5090d8c78`（R3-01 `DONE` 后开放 R3-02）
- 当前已验证实现提交：`cd754f79456b96b3e66ac45b119f61609346e06d`
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

`src-tauri/src/commands/database.rs` 不再直接连接 PostgreSQL 或调用 `reset_to_pristine`。清空流程改为：

`Tauri command -> ApplicationService -> DatabaseService -> reset use case -> DatabaseLifecyclePort -> PostgreSQL adapter`

Tauri 仍保留既有配置读取、运行日志、P4 worker 恢复和返回 DTO 语义。

### 5. 旧实现清理

删除 `crates/application/src/database.rs`。数据库职责不再由该根级混合文件承担，当前唯一入口已切换到 `services/database/` 与 `use_cases/database/`。

## 编译失败诊断与修复

第一轮实施验证在 Database Service 专项与架构门禁通过后，`cargo check --locked -p football-application` 暴露两个真实兼容问题：

1. `active_store` 随文件从 crate 根子模块迁入 `services::database::facade` 后仍使用 `pub(super)`，导致可见范围意外缩小，多个 crate 内既有调用方出现 `E0624`。已改为 `pub(crate)`，恢复旧实现实际具备的 crate 内访问范围，不扩大外部公共 API。
2. `recover_interrupted_api_workspace_operations()` 返回恢复数量，新的 `recover_interrupted_work()` 误把 `Result<u64, _>` 作为 `Result<(), _>` 返回。已显式丢弃计数并返回 `Ok(())`，恢复旧流程的 `()` 语义。

同时移除 `crates/application/src/lib.rs` 中两个已无调用的 crate-private 根别名，避免后续 Clippy `-D warnings` 因未使用导入失败。没有增加 `allow`、跳过检查或降低门禁。

## 文件清单

与 R3-02 起点 `7194a46b520c23aaeedef3aa785b97a5090d8c78` 相比，当前节点涉及：

### 新增

- `crates/application/src/services/database/bootstrap.rs`
- `crates/application/src/services/database/facade.rs`
- `crates/application/src/services/database/mod.rs`
- `crates/application/src/services/database/service.rs`
- `crates/application/src/services/mod.rs`
- `crates/application/src/use_cases/database/connect/mod.rs`
- `crates/application/src/use_cases/database/health/mod.rs`
- `crates/application/src/use_cases/database/migrate/mod.rs`
- `crates/application/src/use_cases/database/mod.rs`
- `crates/application/src/use_cases/database/reset/mod.rs`
- `crates/application/src/use_cases/database/statistics/mod.rs`
- `crates/application/src/use_cases/mod.rs`
- `scripts/verify-database-service.mjs`
- 本记录文件。

### 修改

- `architecture/domain-type-inventory.json`（由确定性生成器同步源码扫描元数据）
- `architecture/state-ownership.json`
- `crates/application/src/composition/application_composition.rs`
- `crates/application/src/composition/mod.rs`
- `crates/application/src/composition/port_registry.rs`
- `crates/application/src/lib.rs`
- `crates/application/src/ports/database/mod.rs`
- `crates/application/src/service/application_service.rs`
- `package.json`
- `scripts/verify-application-composition.mjs`
- `scripts/verify-database-reset.mjs`
- `scripts/verify-frontend.mjs`
- `src-tauri/src/commands/database.rs`

### 删除

- `crates/application/src/database.rs`

### 移动/重命名

- 无。旧职责被重写到具名模块后删除，没有保留 `old/new/legacy/copy/final/v2` 文件。

## 契约与行为

- `ApplicationService` 既有数据库公开方法名称与调用语义：保持。
- Bootstrap / DatabaseHealth / DatabaseStats 数据结构：保持。
- 数据库配置键与保存格式：保持。
- reset 数据库名称确认与错误提示语义：保持。
- PostgreSQL Schema、迁移 SQL、历史数据格式：未修改。
- Tauri 公共命令与 DTO：未修改结构。
- 模型 API、P4/P7 保护资产：未修改。
- 生产依赖与 `Cargo.lock`：未新增或升级。

## 验证记录

### 实施侧 Windows 专项

Temporary R3-02 Validate run `31244006019` / job `93069490517` 在实现提交 `d35029a8669a3e8a1e69716ff8840cdcf4413c5d` 上完成，并生成支持更新提交 `cd754f79456b96b3e66ac45b119f61609346e06d`。

已通过：

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

该 run 的架构、Application check 和 Application tests 均为 `success`；临时 workflow 已在成功提交中删除。

### 尚未完成

以下仍需以用户 Windows 本机为本节点验收依据：

- R3-02 最小验证复跑。
- `npm run verify:frontend`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。
- `npm run tauri:dev` 数据库连接/bootstrap 运行时烟测。

真实 PostgreSQL destructive reset 不使用用户原有数据库做验证；如后续需要执行 destructive integration，只允许使用名称与环境均满足项目安全门禁的专用测试数据库。

## 回退

节点源码可回退到 R3-02 起点 `7194a46b520c23aaeedef3aa785b97a5090d8c78`。不得通过复制恢复已删除的 `database.rs`，也不得绕开 Ports/Service/Use Case 边界恢复直接 PostgreSQL 调用。

## 下一步

状态保持 `VERIFYING`。Windows 本机最小验证、阶段回归、运行时烟测和 README 同步全部完成后，才能将 R3-02 改为 `DONE` 并开放 R3-03。
