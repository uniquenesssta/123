# R04 阶段完成 / 出口验证记录

- 阶段状态：`VERIFYING`
- R4 起点：`f2e4841aac873f6a4812801e6be2a6524cd680c1`
- R4-04 code merge commit：`b97587c9d20165018f80040dc2a2c098dbbec177`
- 目标平台：Windows

## 1. 阶段目标与完成结论

R4-01 至 R4-04 的代码重写节点均已完成并关闭为 `DONE`：Persistence Store/Error/Pool/Migration/Health/Statistics、Audit、基础 Row Mapping 与 Application Port Adapter 注册均已收敛到职责模块，旧职责 owner/转发层已按节点范围清理。

**R4 整体阶段当前不能标记为 `DONE`。** 任务书“阶段级验证矩阵”把空库 migration、health/stats integration、audit transaction integration 标为必须执行；当前无专用可写 `FOOTBALL_TEST_DATABASE_URL`，18 个 PostgreSQL integration tests 保持 ignored，且未执行 destructive database reset。静态数据库冻结、crate/workspace tests 与 Windows Automated 不替代该真实数据库门禁。因此本记录作为阶段出口验证记录存在，阶段状态保持 `VERIFYING`，R5-01 继续 `BLOCKED`。

## 2. 已完成节点索引

| 任务 ID | 实施记录 | 节点状态 | 关键验证 |
|---|---|---|---|
| R4-01 | [`R04-01-store-error-and-pool.md`](./R04-01-store-error-and-pool.md) | DONE | hard gate `31615481637`; clean CI `31615930113` |
| R4-02 | [`R04-02-audit-基础设施.md`](./R04-02-audit-基础设施.md) | DONE | hard gate `31629430245`; clean CI `31630618699` |
| R4-03 | [`R04-03-row-映射基础规范.md`](./R04-03-row-映射基础规范.md) | DONE | hard gate `31674700550`; clean CI `31675727990`; stage CI `31677600876` |
| R4-04 | [`R04-04-port-adapter-注册.md`](./R04-04-port-adapter-注册.md) | DONE | hard gate `31712193150`; PR CI `31717901248`; stage CI `31724131556` |

## 3. 实际新增文件总表

| 文件 |
|---|
| `docs/modular-rewrite/R04-persistence-foundation/R04-02-audit-基础设施.md` |
| `docs/modular-rewrite/R04-persistence-foundation/R04-03-row-映射基础规范.md` |
| `docs/modular-rewrite/R04-persistence-foundation/R04-04-port-adapter-注册.md` |
| `crates/application/src/composition/adapters/competition.rs` |
| `crates/application/src/composition/adapters/database.rs` |
| `crates/application/src/composition/adapters/persistence_error.rs` |
| `crates/application/src/composition/adapters/rules.rs` |
| `crates/persistence-postgres/src/adapters/mod.rs` |
| `crates/persistence-postgres/src/adapters/register_adapters.rs` |
| `crates/persistence-postgres/src/audit/audit_event.rs` |
| `crates/persistence-postgres/src/audit/audit_hash.rs` |
| `crates/persistence-postgres/src/audit/audit_payload.rs` |
| `crates/persistence-postgres/src/audit/mod.rs` |
| `crates/persistence-postgres/src/audit/write_audit_event.rs` |
| `crates/persistence-postgres/src/competition_kind.rs` |
| `crates/persistence-postgres/src/error/mod.rs` |
| `crates/persistence-postgres/src/error/persistence_error.rs` |
| `crates/persistence-postgres/src/health/database_health.rs` |
| `crates/persistence-postgres/src/health/mod.rs` |
| `crates/persistence-postgres/src/health/read_health.rs` |
| `crates/persistence-postgres/src/mapping/invalid_state.rs` |
| `crates/persistence-postgres/src/mapping/json.rs` |
| `crates/persistence-postgres/src/mapping/mod.rs` |
| `crates/persistence-postgres/src/mapping/optional.rs` |
| `crates/persistence-postgres/src/mapping/time.rs` |
| `crates/persistence-postgres/src/mapping/uuid.rs` |
| `crates/persistence-postgres/src/migrations/mod.rs` |
| `crates/persistence-postgres/src/migrations/reset_to_pristine.rs` |
| `crates/persistence-postgres/src/migrations/run_migrations.rs` |
| `crates/persistence-postgres/src/migrations/runtime_schema.rs` |
| `crates/persistence-postgres/src/pool/create_pool.rs` |
| `crates/persistence-postgres/src/pool/database_options.rs` |
| `crates/persistence-postgres/src/pool/mod.rs` |
| `crates/persistence-postgres/src/statistics/database_stats.rs` |
| `crates/persistence-postgres/src/statistics/mod.rs` |
| `crates/persistence-postgres/src/statistics/read_statistics.rs` |
| `crates/persistence-postgres/src/store/mod.rs` |
| `crates/persistence-postgres/src/store/postgres_store.rs` |
| `docs/modular-rewrite/R04-persistence-foundation/R04-01-store-error-and-pool.md` |
| `docs/modular-rewrite/R04-persistence-foundation/R04-stage-completion.md` |
| `docs/modular-rewrite/R04-persistence-foundation/README.md` |
| `scripts/verify-persistence-adapters.mjs` |
| `scripts/verify-persistence-audit.mjs` |
| `scripts/verify-persistence-foundation.mjs` |
| `scripts/verify-persistence-mapping.mjs` |

## 4. 实际修改文件总表

| 文件 |
|---|
| `README.md` |
| `architecture/database-baseline.json` |
| `architecture/domain-type-inventory.json` |
| `architecture/module-boundaries.json` |
| `architecture/state-ownership.json` |
| `crates/application/src/composition/adapters/ai_workspace.rs` |
| `crates/application/src/composition/adapters/analytics.rs` |
| `crates/application/src/composition/adapters/exchange/match_lineup.rs` |
| `crates/application/src/composition/adapters/exchange/spreadsheet.rs` |
| `crates/application/src/composition/adapters/jobs.rs` |
| `crates/application/src/composition/adapters/lineups.rs` |
| `crates/application/src/composition/adapters/mod.rs` |
| `crates/application/src/composition/adapters/players.rs` |
| `crates/application/src/composition/adapters/postmatch.rs` |
| `crates/application/src/composition/adapters/prediction.rs` |
| `crates/application/src/composition/adapters/release.rs` |
| `crates/application/src/composition/adapters/research.rs` |
| `crates/application/src/composition/adapters/review.rs` |
| `crates/application/src/composition/adapters/teams.rs` |
| `crates/application/src/composition/mod.rs` |
| `crates/application/src/composition/port_registry.rs` |
| `crates/application/src/services/ai_workspace/facade/mod.rs` |
| `crates/application/src/services/analytics/facade.rs` |
| `crates/application/src/services/database/service.rs` |
| `crates/application/src/services/exchange/facade/mod.rs` |
| `crates/application/src/services/lineups/facade.rs` |
| `crates/application/src/services/players/facade.rs` |
| `crates/application/src/services/postmatch/facade.rs` |
| `crates/application/src/services/prediction/facade.rs` |
| `crates/application/src/services/release/facade.rs` |
| `crates/application/src/services/research/facade.rs` |
| `crates/application/src/services/review/facade.rs` |
| `crates/application/src/services/teams/facade.rs` |
| `crates/application/src/use_cases/application_facade/database_lifecycle/initialize.rs` |
| `crates/persistence-postgres/src/lib.rs` |
| `crates/persistence-postgres/src/match_exchange.rs` |
| `crates/persistence-postgres/src/model_runs.rs` |
| `crates/persistence-postgres/src/player_catalog.rs` |
| `crates/persistence-postgres/src/spreadsheet_exchange.rs` |
| `crates/persistence-postgres/src/team_catalog.rs` |
| `crates/persistence-postgres/src/team_force_delete.rs` |
| `package.json` |
| `scripts/verify-ai-workspace-service.mjs` |
| `scripts/verify-analytics-service.mjs` |
| `scripts/verify-application-composition.mjs` |
| `scripts/verify-competition-rules-service.mjs` |
| `scripts/verify-database-migration-compatibility.mjs` |
| `scripts/verify-database-reset.mjs` |
| `scripts/verify-database-service.mjs` |
| `scripts/verify-exchange-service.mjs` |
| `scripts/verify-lineups-service.mjs` |
| `scripts/verify-prediction-service.mjs` |
| `scripts/verify-release-service.mjs` |
| `scripts/verify-research-service.mjs` |
| `scripts/verify-review-service.mjs` |
| `scripts/verify-teams-players-service.mjs` |

## 5. 实际移动或重命名文件总表

| 原文件 | 新文件 | Git 状态 |
|---|---|---|
| `crates/persistence-postgres/src/migration_compatibility.rs` | `crates/persistence-postgres/src/migrations/reconcile_known_migrations.rs` | `R090` |

## 6. 实际删除文件总表

| 文件 |
|---|
| `crates/persistence-postgres/src/connection.rs` |

## 7. 最终目录与职责边界

- `store/`：`PostgresStore` 唯一连接池 owner；R4-04 补充 redacted URL 元数据，不引入新的共享状态 owner。
- `pool/`：DatabaseOptions 与 SQLx pool 创建。
- `migrations/`：migration runner、已知迁移兼容、runtime schema、destructive reset 各自独立。
- `health/` / `statistics/`：数据库健康与统计读取独立职责。
- `error/`：PersistenceError/PersistenceResult 唯一 owner。
- `audit/`：事件、payload、hash、SQL writer 分责；业务写与 audit writer 继续由事务调用方控制原子边界。
- `mapping/`：time/UUID/JSON/optional/invalid-state 基础标量映射；业务 enum 不进入通用 mapper。
- `adapters/`：Persistence 只拥有注册入口；Application-owned Port trait impl 留在 `crates/application/src/composition/adapters/`，concrete target 为 `PostgresStore`，避免 crate cycle。
- Application composition 只在边界持有 concrete persistence；services/use-cases 通过 crate-private `DatabaseSession` 边界保持不泄漏 PostgreSQL 实现。

## 8. 最终调用流、数据流和状态所有权

`Application Port -> Application composition adapter impl for PostgresStore -> persistence method/query -> SQLx -> Row -> mapping/domain`。

连接池状态只由 `PostgresStore` 持有；业务 Service/Use Case 不拥有 PgPool。R4-04 删除 `ActiveDatabase` 运行时 wrapper 和 `transition_store()` 转发链，不建立重复数据库状态。

## 9. 公共接口、DTO、Schema、数据与配置变化

- 公共 Application Ports trait 名称/调用语义：未改变。
- Tauri command、参数、DTO 与前端调用契约：未改变。
- PostgreSQL schema、历史 0001–0046 migration SQL、持久化数据格式：未改变。
- 配置键、默认值、环境变量、生产依赖：未改变。
- 公共错误类别/错误传播语义与日志等级：未改变。
- 模型实现、参数、Profile、Schema/fixture/Golden 与模型保护资产：未修改。

## 10. 保持不变的兼容行为

- `PostgresStore::connect` 继续保留给既有低层调用和 PostgreSQL integration tests。
- DatabaseOptions Serde、health/stats 输出、migration compatibility、audit hash 算法、Row 基础解析错误语义保持。
- CompetitionKind 业务解析保持独立 owner，不被通用 mapping 吞并。
- ApplicationService/Tauri/前端用户可观察行为没有因 R4 重写改变。

## 11. 旧实现、重复实现和临时路径清理结果

- 旧 `crates/persistence-postgres/src/connection.rs` 已删除。
- `migration_compatibility.rs` 已迁入具名 `migrations/reconcile_known_migrations.rs`。
- lib.rs 中重复 Store/Error/Audit/Mapping 业务 owner 已清理。
- R4-02 清理重复/直写 audit INSERT；R4-04 清理 `ActiveDatabase` 与 `transition_store()`。
- R4-01~04 临时 workflow/helper 未进入 canonical stage tree；R4-04 assistant-created `.noop` / marker 在最终 PR tree 中为零差异。
- 未建立 `mapper.rs`、`queries.rs`、万能 Repository 或无退出计划兼容层。

## 12. 阶段级验证与真实结果

- R4-01 hard gate `31615481637`；clean Public Platform CI `31615930113` / job `94178594481`：SUCCESS。
- R4-02 hard gate `31629430245`；clean Public Platform CI `31630618699` / job `94228122818`：SUCCESS。
- R4-03 hard gate `31674700550`；PR CI `31675727990` / job `94369762967`、post-merge stage CI `31677600876` / job `94375513281`：SUCCESS。
- R4-04 final hard gate `31712193150`：R4-01~04 专项、数据库静态 baseline/protected assets、rustfmt、Persistence/Application check/tests、完整 architecture/frontend、workspace Clippy `-D warnings` 与 workspace tests 全部 PASS；Persistence 80/80 unit tests、Application 33/33 tests PASS。
- R4-04 PR #25 clean Public Platform CI `31717901248` / job `94507174688`：SUCCESS；artifact `9188959652`，SHA-256 `477b32b66fb06ee9bfa04ead7778639f1d5658f9478a641c706416c49b70d687`。
- R4-04 squash merge `b97587c9d20165018f80040dc2a2c098dbbec177` 后 canonical stage Public Platform CI `31724131556` / job `94528167665`：SUCCESS；artifact `9191415520`，SHA-256 `3ccb37d8eab17c0e589354c10c3423579397629f9f76481b267acd0749db38cf`。
- Cargo manifests / `Cargo.lock` 与历史 migrations 对节点基线保持冻结；R4 阶段总 diff 未触及 `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/`。

## 13. 未执行验证、环境阻塞和剩余风险

未执行且不得描述为通过：

- 18 个要求专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL integration tests。
- 空库真实 migration 0001–0046 执行链。
- 真实 PostgreSQL health/stats integration。
- 真实 PostgreSQL audit transaction integration。
- destructive database reset。

原因：当前执行环境没有经确认可彻底清空的专用测试数据库；项目安全规则禁止对非 test 数据库执行破坏性验证。替代验证已完成静态 migration 指纹/连续性、SQLx 入口、crate/workspace tests、architecture 与 Windows Automated，但这些不替代任务书的真实 PostgreSQL stage gate。

剩余风险：R4 的 PostgreSQL runtime integration 尚未在真实专用数据库闭环，故 R4 stage 不能进入 `DONE`，R5 不得开始。

## 14. 根 README、阶段 README 与架构文档同步

- 根 `README.md` 已记录 R4-04 实际结果、CI/merge 证据与 R4 stage DB 出口阻塞。
- 本阶段 `README.md` 已将 R4-04 标记 `DONE`、阶段标记 `VERIFYING`，并链接本记录。
- R4-01~04 过程中已按实际 owner 变化同步 `architecture/database-baseline.json`、`domain-type-inventory.json`、`module-boundaries.json`、`state-ownership.json`；未通过 closeout 额外修改这些架构文件。

## 15. 阶段回退点与回退步骤

- R4-04 之前最近完整已验证回退点：`b508d1ff7808b8735694cf1e57a4d603f2e973e3`。
- R4 全阶段开始前回退点：`f2e4841aac873f6a4812801e6be2a6524cd680c1`。
- R4-04 当前 canonical code merge：`b97587c9d20165018f80040dc2a2c098dbbec177`。
- 如需回退，使用 Git 提交级回退到上述已验证点；不得手工复制旧文件、恢复双实现或修改历史 migration。

## 16. 出口门禁逐项结论

| 出口项 | 结论 |
|---|---|
| R4-01~R4-04 节点记录齐全且节点完成 | PASS |
| Persistence 基础模块职责边界与 dependency direction | PASS |
| 历史 migration 内容未改变 / 静态冻结 | PASS |
| architecture/frontend/workspace Rust/Windows Automated | PASS |
| 空库 migration 真实执行 | **BLOCKED / 未执行** |
| health/stats PostgreSQL integration | **BLOCKED / 未执行** |
| audit transaction PostgreSQL integration | **BLOCKED / 未执行** |
| R4 stage 状态 | **VERIFYING，不得标记 DONE** |

## 17. 下一阶段唯一 READY 任务

无。R5-01 `Competitions Repository` 是任务书中的下一任务，但在 R4 阶段真实 PostgreSQL 出口门禁补齐前继续 `BLOCKED`；本记录不创建 R5 生产实现或预迁移代码。

## 18. 订正记录

- docs-only closeout 第二次 run `31727494199` 已成功生成并发布四份收口文档到 stage commit `837d364cbddb174f794529efb4b7be76fa673f5e`；随后仅 transient helper 自清理因 runner 内临时改写 helper 导致 `git rm` 拒绝而失败，未影响 canonical stage。assistant-created helper/workflow 已随后通过 connector 提交 `dc992b7630f1559c1aaa0e10fd2a30626450d7da` / `acc9884277355410a1f960ee91c0bcc2e8f4b634` 从 helper 分支删除；canonical stage 从未包含这些 transient 文件。
- docs-only closeout 首次 run `31727111563` 在发布前 fail-fast：scope 校验仅读取 tracked `git diff`，遗漏新建且仍 untracked 的 `R04-stage-completion.md`；stage 发布和 transient cleanup 步骤均被跳过，canonical stage 未发生变化。恢复仅将 scope 校验改为 tracked diff 与 untracked 文件的精确并集，最终允许集合仍严格为四份收口文档。
- R4-04 节点在 hard gate、clean PR CI、squash merge 与 post-merge stage CI 全部成功后关闭为 `DONE`。
- 阶段收口时重新按 R4 任务书检查“阶段级验证矩阵”，确认 18 个 PostgreSQL integration tests / 空库 migration / health-stats / audit integration 仍未执行；因此没有沿用“节点全部 DONE 即阶段 DONE”的简化判断，而是将 R4 stage 保持 `VERIFYING`。该订正只影响阶段状态和文档，不修改生产源码、接口、数据或验证门禁。
