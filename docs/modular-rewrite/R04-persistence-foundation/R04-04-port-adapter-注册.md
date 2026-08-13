# R4-04 Port Adapter 注册

## 状态

`DONE`

## 实施结果

- 在 `crates/persistence-postgres/src/adapters/` 建立 `mod.rs` 与 `register_adapters.rs`，由 Persistence 暴露唯一 Application composition 注册入口；`PostgresStore::connect` 继续保留给现有低层兼容调用和 PostgreSQL integration tests。
- `PostgresStore` 接管活动连接的 redacted URL 元数据，删除 Application `ActiveDatabase` wrapper 与 `transition_store()` 转发链。
- 40 个 PostgreSQL-backed Application Port 实现目标全部从 `ActiveDatabase` 切换为 `PostgresStore`（Application crate 持有 trait，因此 impl 保持在 Application composition adapters，避免反向依赖和 crate cycle）。
- `port_registry.rs` 收敛为 concrete persistence import + `register_adapters` 连接注册；原 Database / Competition / Rules Port 实现拆到 `composition/adapters/database.rs`、`competition.rs`、`rules.rs`，PersistenceError -> PortError 映射拆到 `persistence_error.rs`。
- 新增 `verify:persistence-adapters` 并接入 `verify:architecture`，锁定唯一 concrete import owner、40-Port target、零 `ActiveDatabase`/`transition_store`、注册入口与模块边界。
- 未修改 Cargo dependency graph、公共 Application Ports、Tauri command/DTO、SQL、0001–0046 migrations、配置格式、错误类别语义、模型保护资产或 R5 业务 Repository。

## 审计依据

- scope audit run `31686092182`：确认 Application 只有 `composition/port_registry.rs` 直接导入 `football_persistence_postgres`；40 个 DB-backed Port impl 全部以 `ActiveDatabase` 为旧目标；`PostgresStore::connect` 另有现有 integration test 调用；Persistence crate 不依赖 Application，直接反转依赖会形成任务书禁止的 cycle。

## 验证

- 第九次 implementation run `31689306408`：候选 rewrite、11 个同源旧 owner verifier 迁移、最小硬门禁均通过，`npm run verify:architecture` 全链 PASS；随后 `npm run verify:frontend` fail-fast，workspace Clippy/tests、frozen/docs/commit 未执行，远端仍无生产源码提交。
- frontend diagnostic run `31689940421` 精确定位失败为 `verify-teams-players-service.mjs`：机械地把 composition 外原 `ActiveDatabase` 会话边界替换成 `PersistenceStore`，导致 Teams/Players facade 泄漏 concrete persistence。该 verifier 是有效架构门禁，未删除或放宽。
- session-boundary audit run `31690075828` 枚举基线 composition 外全部 20 处 `ActiveDatabase` 引用，并确认多项 Service verifier 明确禁止 `PersistenceStore`/`PostgresStore`/`football_persistence_postgres` 泄漏。恢复方案建立 crate-private、零运行时包装的 `DatabaseSession = PersistenceStore` type alias：services/use-cases 只依赖 `DatabaseSession`，40 个 Application Port impl 仍直接 `for PersistenceStore`，旧 `ActiveDatabase` struct 与 `transition_store` 不恢复。
- full frontend diagnostic run `31690313018` 使用与主 hard gate 相同候选树生成链并加入 `DatabaseSession` 边界后，`npm run verify:persistence-adapters` 与完整 `npm run verify:frontend` 均 PASS。该诊断未替代 workspace Clippy/tests；最终 hard gate 仍需从头执行完整最小门禁与 stage regression。

- 首次 implementation run `31686470233`：起点保护范围通过；实施 helper 在切换 Application adapters 时发现 `analytics.rs` 存在多行 `self\n.transition_store()` 形式，生成器只覆盖单行形式后主动 fail-fast。最小门禁、阶段回归、文档与提交步骤均未执行，远端未产生任何生产源码提交。恢复 helper 改为同时规范单行/多行转发，并从基线重新生成全部 adapter 变更。
- 第二次 implementation run `31686662626`：起点保护范围与规范化 adapter 生成已完成，`write_verifier_and_package()` 也实际成功；临时 runner 却因控制流保护错误，把“无需 fallback”误判为异常并主动 fail-fast。最小门禁、阶段回归、文档与提交步骤仍未执行，远端仍无生产源码提交。恢复只移除该错误 guard，成功路径直接继续，fallback 仍仅捕获唯一已知 package anchor mismatch，其他异常继续硬失败。
- 第三次 implementation run `31686951586`：Apply 已成功并首次进入真正最小门禁；`verify:persistence-adapters` 先行失败，原因是 verifier 用单行字符串匹配 `register_adapters(options).await.map_err(...)`，而 rustfmt 将相同调用链格式化为多行。该 run 在专项静态门禁即 fail-fast，R4-01/R4-02/R4-03 后续门禁、Rust compile/tests、stage regression、文档与提交均未执行。恢复仅将此断言改为格式无关正则，不修改生产注册实现、不降低语义检查。
- 第四次 implementation run `31687122419`：R4-04 专项及 R4-01/R4-02/R4-03 回归均已通过；数据库 baseline 随后检测到本节点合法修改的 `crates/persistence-postgres/src/store/postgres_store.rs` runtime-source Git blob 指纹从 `939445bec1e0e35ab28c25173d87e2ccf4dc79de` 变化而 fail-fast。Rust compile/tests 与 stage regression 尚未执行，远端仍无生产源码提交。R4-01 已建立对合法 Persistence runtime 重构同步 `architecture/database-baseline.json` 的先例；恢复仅在 rustfmt 后重新计算并替换该单一 runtime-source 指纹，46 个 migration SQL、migration aggregate、integration test 集合和其他 runtime-source 指纹保持冻结。
- 第五次 implementation run `31687372054`：R4-04 专项、R4-01/R4-02/R4-03 回归、数据库 baseline、保护资产、rustfmt 均已通过，并首次进入 `cargo check -p football-persistence-postgres -p football-application`。Application 编译暴露 4 个 wrapper 移除后的解析边界错误：`map_persistence_error` 仅为 adapters-parent 可见，不能供 sibling `port_registry` 使用；以及 `PostgresStore` inherent `close() -> ()` 优先于 `DatabaseLifecyclePort::close() -> PortResult<()>`，使 `activate/disconnect` 两处 `?` 无法应用。Persistence/Application tests 与 stage regression 因 compile fail-fast 未执行，远端仍无生产源码提交。恢复将 error mapper 精确限制为 `pub(in crate::composition)`，并在 `DatabaseService` 两个需要传播结果的关闭路径显式调用 `DatabaseLifecyclePort::close(&store)`，保持原 PortResult/错误传播语义；专项 verifier 同步锁定这两个兼容约束。
- 第六次 implementation run `31687732166`：R4-04 最小硬门禁完整通过：R4-04/R4-01/R4-02/R4-03 专项、数据库 baseline/保护资产、rustfmt、Persistence+Application `cargo check`、Persistence 80/80 unit tests、Application 33/33 tests 均通过；18 个 PostgreSQL integration tests 仍按契约为 ignored，未执行。阶段回归随后执行 `npm run setup` 并进入 `verify:architecture`，模块边界、状态所有权、受保护导入、Domain inventory/root exports、Application Ports 均通过，但旧 `verify-database-service.mjs` 仍以 `ActiveDatabase` 字面类型和旧 `port_registry.rs` impl 位置断言 DatabaseService/DatabaseLifecyclePort/DatabaseObservabilityPort，因 R4-04 合法 owner 迁移而 fail-fast；`verify:frontend`、workspace Clippy/tests 未执行，远端仍无生产源码提交。恢复不删除或放宽 Database Service gate，而是将其精确迁移到 `DatabaseService.session: PersistenceStore`、`composition/adapters/database.rs` 的 Lifecycle/Observability impl 与 `port_registry.rs` 的 `register_adapters` composition 入口，并继续拒绝 ApplicationService 直接持有活动数据库槽位。
- 第七次 implementation run `31688354846`：最小硬门禁再次完整通过，迁移后的 Database Service gate 也已通过；`verify:architecture` 继续到 Competition/Rules Service 时，旧 `verify-competition-rules-service.mjs` 仍要求三个 Port `for ActiveDatabase` 且位于 `port_registry.rs`，因此 fail-fast。`verify:frontend`、workspace Clippy/tests、frozen/docs/commit 未执行，远端仍无生产源码提交。为避免同类旧 owner 假设逐个串行暴露，追加 scope audit run `31688840100`，一次性扫描 `scripts/`：除已迁移 Database gate 外，共 11 个 verifier 仍含 `ActiveDatabase` 正向 adapter-owner 断言。恢复使用严格 allowlist 同步迁移这 11 个 owner 断言到 `PersistenceStore`/现有具名 adapter 文件；service/use-case 中禁止 `PersistenceStore`/`football_persistence_postgres` 泄漏以及禁止 `transition_store` 的负向约束全部保持不变。
- 第八次 implementation run `31689104070`：起点范围与 R4-04 生产 rewrite 均生成成功，但新的 verifier migrator 在执行后自检时将 R4-04 专项中的负向保护 `check(!adapterCombined.includes("for ActiveDatabase"))` 误判成“残留正向旧 owner”并主动 fail-fast。最小门禁、阶段回归、文档与提交均未执行，远端仍无生产源码提交。恢复仅让迁移器排除 `verify-persistence-adapters.mjs` 这份故意保留旧 token 的负向拒绝门禁；其他 verifier 的正向 `ActiveDatabase` owner 假设仍必须清零。

Windows 2025 / Rust 1.88.0 / Node 22 hard gate run `31712193150` 实际执行并通过：

- `npm run verify:persistence-adapters`。
- `npm run verify:persistence-foundation`。
- `npm run verify:persistence-audit`。
- `npm run verify:persistence-mapping`。
- `node scripts/verify_database_baseline.mjs`。
- `node scripts/verify_protected_assets.mjs`。
- `cargo fmt --all -- --check`。
- `cargo check --locked -p football-persistence-postgres -p football-application`。
- `cargo test --locked -p football-persistence-postgres`。
- `cargo test --locked -p football-application`。
- `npm run setup`。
- `npm run verify:architecture`。
- `npm run verify:frontend`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。
- Cargo manifests / `Cargo.lock`、历史 migrations 对 R4-04 基线 `b508d1ff7808b8735694cf1e57a4d603f2e973e3` 零 diff。

## 正式收口

- final implementation hard gate run `31712193150`：`SUCCESS`；R4-04/R4-01/R4-02/R4-03 专项、数据库静态冻结、保护资产、rustfmt、Persistence/Application compile/tests、完整 architecture/frontend、workspace Clippy `-D warnings` 与 workspace tests 均通过。
- PR #25 clean Public Platform CI run `31717901248` / Windows automated delivery job `94507174688`：`SUCCESS`；artifact `9188959652`，13,894,348 bytes，SHA-256 `477b32b66fb06ee9bfa04ead7778639f1d5658f9478a641c706416c49b70d687`。
- 最终 PR HEAD `b808ff3cc7a43f3c68b0f024228adefb20b916cb` 的 tree 与已验证 implementation `a7f1dc852bc486dbf0b1ca4dd7db5d262950d7c8` 文件内容零差异；用户授权清理的 `.noop` 及 assistant-created marker 均不在最终 PR/stage tree。
- PR #25 按固定 HEAD `b808ff3cc7a43f3c68b0f024228adefb20b916cb` squash merge；stage merge commit `b97587c9d20165018f80040dc2a2c098dbbec177`。
- 合并后 canonical stage Public Platform CI run `31724131556` / job `94528167665`：`SUCCESS`；artifact `9191415520`，13,895,272 bytes，SHA-256 `3ccb37d8eab17c0e589354c10c3423579397629f9f76481b267acd0749db38cf`。
- R4-04 节点正式关闭为 `DONE`；未提前实施 R5-01。

## 未执行与剩余风险

- R4-04 节点自身已 `DONE`；后续数据库阶段验证不改变其生产实现与兼容性结论。
- broad PostgreSQL diagnostic `31728096953`：18 个 ignored integration tests 实跑 14/18 PASS，destructive reset PASS；3 个失败为既有过期业务夹具，另 1 个为 R4 排除范围内的既有 P4 timestamp 精度问题，未通过放宽生产规则追求 18/18。
- final R4 专项 PostgreSQL stage gate `31729577225` / job `94546316946`：空库 0001–0046、health、stats、audit 失败回滚与成功事务提交均 PASS；首次 scoped `31729361081` 仅因 runner connection-local trigger 设计错误失败，未产生生产源码改动。
- R4 stage 因真实数据库矩阵补齐而正式 `DONE`；R5-01 仅开放为 `READY`，本节点未提前实现 R5。

## 回退点

- R4-04 基线：`b508d1ff7808b8735694cf1e57a4d603f2e973e3`。
- 实施分支：`agent/r4-04-port-adapter-registration`。
