# R05-01 — Competitions Repository

## 状态

`VERIFYING`

实现、契约冻结、最小门禁、架构回归、Windows 前端与 Windows workspace Rust 回归均已完成。当前仅等待 clean Pull Request 的 Public Platform CI、合并到 `rewrite/r5-competition-routing-persistence` 以及合并后 stage CI；在这些证据完成前不标记为 `DONE`，R5-02 保持 `BLOCKED`。

## 基线与分支

- R4 最终 closeout HEAD：`615dc952491d5e0e21d4979292cbf5170eedece6`。
- 该 HEAD 的 canonical Public Platform CI：run `31768264193` / Windows automated delivery gate job `94668577704`，`SUCCESS`。
- R5 stage 分支：`rewrite/r5-competition-routing-persistence`，从上述 HEAD 精确建立。
- R5-01 实施分支：`agent/r5-01-competitions-repository`，从上述 HEAD 精确建立。

## 实际问题与边界

R5-01 开工前，`crates/persistence-postgres/src/competitions.rs` 同时承担 Competition 目录/详情 CRUD、Season/Stage/Round 层级、scope context 解析和动态 Row 映射，职责边界不清晰。Application 已冻结的 Competition Port 中，本节点只迁移：

- `create_competition`
- `read_competition`
- `list_competitions`
- `delete_competition`

`create/list season/stage/round` 与 `resolve_competition_context` 明确保留给 R5-02 / R5-05；Rule Package、Binding、Route Resolution、Model Run Identity 均未提前实施。

## 实际实现

- 新建 `adapters/competition/directory/`：Competition create/list/delete 目录职责。
- 新建 `adapters/competition/detail/`：Competition detail read、typed Row、Domain Mapper。
- 每个 SQL 文件只表达一个查询/写入目的。
- soft delete 的多表写入拆为 `delete_competition/` 子目录，由具名 `transaction.rs` 只负责事务编排；锁定 active competition、停用 bindings、删除 external entity IDs、soft-delete competition 分别独立。
- Competition Row 从旧动态 `PgRow` 映射迁为 typed `sqlx::FromRow` Row，并与 Domain Mapper 分离；未新增通用万能 mapper。
- `crates/persistence-postgres/src/competitions.rs` 删除本节点已迁移的 Competition CRUD/Row mapping，只保留 R5-02/R5-05 尚未迁移职责；不保留转发壳。
- 新增 `scripts/verify-competition-repository.mjs` 并接入 `verify:architecture`，锁定唯一 owner、typed Row/Mapper、单 SQL 职责及“不提前迁移 R5-02/R5-05”。
- `architecture/domain-type-inventory.json` 使用项目既有生成器刷新；Domain 类型数 365、Domain source 129、PostgreSQL mapping type 299 均不变，Rust 扫描文件数随新职责模块增加，caller/mapping owner 从旧 `competitions.rs` 精确迁到新 directory/detail 文件。

## 兼容性

以下均未改变：

- Application Competition Port 公共方法、参数、返回类型与调用语义。
- competition/stage/round ID 与现有绑定关系。
- 数据库 Schema、0001–0046 migration、SQL 数据格式与历史数据兼容路径。
- 错误类型、错误语义、日志等级、配置、环境变量。
- Tauri 公共命令、DTO、前端赛事页面与用户可观察行为。
- 路由算法、route result、model identity。
- Cargo manifests、Cargo.lock 与生产依赖。
- `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 及模型保护资产。

## PostgreSQL 契约冻结

先于生产 owner 切换新增 `tests/competitions_repository_contract.rs`，并在旧实现基线上用临时 PostgreSQL 16 测试库执行：

- run `31772436658` / job `94680947724`：`SUCCESS`。
- 锁定 create/read/list/delete、name/code trim、typed Domain 字段与 soft-delete 数据事实。

切换新 Repository 后，同一 contract 在 stage diagnostic 中再次显式运行并 1/1 `PASS`。

## 验证记录

### 最终 Ubuntu stage diagnostic

run `31773549494` / job `94684193535`：workflow 总结为 `failure`，但失败点必须按步骤区分：

- R5-01 ownership + R4 persistence regression：`SUCCESS`。
- frozen database + protected assets：`SUCCESS`。
- `cargo fmt --all -- --check`：`SUCCESS`。
- `cargo check --locked -p football-persistence-postgres -p football-application`：`SUCCESS`。
- Persistence unit tests：80/80 `PASS`。
- Application unit tests：33/33 `PASS`。
- `competitions_repository_contract`（专用 PostgreSQL 16）：1/1 `PASS`。
- 完整 `npm run verify:architecture`：`SUCCESS`。
- `npm run verify:frontend` 在截图阶段因 Ubuntu Chromium zygote 无法建立 `DevToolsActivePort` 停止；这是非目标平台浏览器环境阻塞，不记为 frontend 通过。
- 同一 shell 后续 workspace Clippy/tests 因 fail-fast 未执行。

### Cross-platform 补齐

- run `31773777077`：Windows frontend job `94684879652` 在 `windows-2025` 完整 `npm run verify:frontend` 为 `SUCCESS`。
- 同 run 的 Ubuntu workspace Rust job `94684879609` 在项目代码检查前因 runner 缺 `glib-2.0` / `gobject-2.0` 系统库退出 101；不是 Clippy warning，workspace tests 因 fail-fast 未执行，该 job 不记为通过。
- run `31773900831` / Windows workspace Rust job `94685243308`：Rust 1.88.0 下 `cargo clippy --locked --workspace --all-targets -- -D warnings` 与 `cargo test --locked --workspace` 均 `SUCCESS`。

因此本节点要求的最小验证、architecture、保护资产、Windows frontend、workspace Clippy 与 workspace tests 均已有实际通过证据；失败的 Ubuntu browser/system-library 环境记录保留，不用替代验证掩盖。

## 未执行项与剩余风险

- 既有 `postgres_integration.rs` 18 个 ignored PostgreSQL broad integration tests未在 R5-01 执行；本节点只显式执行新增 Competition Repository contract。
- 未执行 destructive database reset。
- 未在用户本机数据库执行写入；所有 PostgreSQL contract 使用临时、可写测试数据库。
- clean PR Public Platform CI、PR merge 与 merged stage Public Platform CI 尚未完成，因此当前状态保持 `VERIFYING`。

## 最终净变更清单（合并前）

### 新增

- `crates/persistence-postgres/src/adapters/competition/detail/mod.rs`
- `crates/persistence-postgres/src/adapters/competition/detail/read_competition.rs`
- `crates/persistence-postgres/src/adapters/competition/detail/record_mapper.rs`
- `crates/persistence-postgres/src/adapters/competition/detail/record_row.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/create_competition.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/deactivate_bindings.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/delete_external_entity_ids.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/lock_active_competition.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/mod.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/soft_delete_competition.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/transaction.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/list_competitions.rs`
- `crates/persistence-postgres/src/adapters/competition/directory/mod.rs`
- `crates/persistence-postgres/src/adapters/competition/mod.rs`
- `crates/persistence-postgres/tests/competitions_repository_contract.rs`
- `scripts/verify-competition-repository.mjs`
- `docs/modular-rewrite/R05-competition-routing-persistence/R05-01-competitions-repository.md`

### 修改

- `architecture/domain-type-inventory.json`
- `crates/persistence-postgres/src/adapters/mod.rs`
- `crates/persistence-postgres/src/competitions.rs`
- `package.json`
- `README.md`
- `docs/modular-rewrite/R05-competition-routing-persistence/README.md`

### 移动/重命名

无。

### 删除

无长期文件。一次性 `.github/workflows/r5-01-contract-baseline.yml` 已在验证完成后删除，不进入最终净 diff。

## 下一状态门禁

只有在 clean PR Public Platform CI 成功、固定 HEAD 合并到 `rewrite/r5-competition-routing-persistence`、合并后 stage Public Platform CI 成功并把证据回填本记录/README 后，R5-01 才能改为 `DONE`，随后才能将 R5-02 改为 `READY`。
