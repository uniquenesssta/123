# R04 阶段完成 / 出口验证记录

- 阶段状态：`DONE`
- R4 起点：`f2e4841aac873f6a4812801e6be2a6524cd680c1`
- R4-04 code merge commit：`b97587c9d20165018f80040dc2a2c098dbbec177`
- 目标平台：Windows

## 1. 阶段目标与完成结论

R4-01 至 R4-04 均已 `DONE`，Store/Error/Pool/Migration/Health/Statistics、Audit、基础 Row Mapping 与 Port Adapter 注册均已收敛到职责模块。

broad PostgreSQL diagnostic `31728096953` 在一次性测试库执行 18 个 ignored tests 为 14/18，destructive reset PASS；4 个失败未掩盖，其中 3 个是当前严格业务契约下的既有过期夹具，1 个是具体 P4 业务持久化 timestamp 精度问题（R4 明确排除范围）。随后独立 R4 stage smoke 最终 run `31729577225` / job `94546316946` 在全新空库真实完成 46 条 migration、health、stats、audit 失败回滚及成功提交，1/1 PASS。architecture dependency 与 migration freeze 亦已通过，因此 R4 整体正式 `DONE`，R5-01 成为唯一 `READY`。

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

- broad PostgreSQL diagnostic `31728096953` / job `94541420957`：PostgreSQL 16 临时测试库真实执行 18 个 ignored tests，14 PASS / 4 FAIL；destructive reset PASS；4 个失败全部留档。
- scoped `31729361081`：空库 migration、health、stats 已通过；audit probe 因 runner 使用 connection-local `set_config` 无法跨 pool connection 触发而失败，属于临时测试设计错误。
- final scoped stage gate `31729577225` / job `94546316946`：Ubuntu 24.04 / PostgreSQL 16.14 / Rust 1.88.0，固定 stage HEAD `2380e440351e3ac6227908ddabfd63e92c49f2e9`；确认初始 schema 为空，迁移 0001–0046 共 46 条 success，health/stats PASS；故意让 `team_updated` audit INSERT 失败并验证业务 update 回滚，再验证成功路径业务 row 与且仅 1 条 audit row 同时提交。runner-only test `1 passed; 0 failed; 0 ignored`。
- R4 阶段出口矩阵：空库 migration PASS；health/stats integration PASS；audit transaction integration PASS；architecture dependency PASS；migration freeze PASS。

## 13. 未执行验证、环境阻塞和剩余风险

R4 任务书要求的真实 PostgreSQL 出口验证已执行，不再存在 R4 foundation 的未执行 stage gate。broad 18-test diagnostic 仍有 4 个失败，不能描述为 18/18：

- `match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent`：旧夹具使用 10 人 confirmed 阵容，与当前 11 人严格规则冲突。
- `match_scope_inference_and_lineup_pair_transaction_are_atomic`：旧夹具的 kickoff / T-6h 快照不在当前窗口。
- `structured_match_events_are_queryable_and_revision_aware`：旧 `result_snapshot` 缺当前 MatchResultRecord 必填字段。
- `p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable`：具体 P4 持久化存在 PostgreSQL timestamptz microsecond 与内存 DateTime 精确比较差异；不在 R4 总 diff，且属 R4 排除范围。

未修改生产规则、跳过测试或跨阶段改 P4 来“修绿”。前三项留给其业务测试/Repository 阶段更新夹具；P4 timestamp 精度问题进入对应业务持久化阶段。临时 DB/service/test/workflow 已清理，未触及用户数据库。

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
| R4-01~R4-04 节点完成 | PASS |
| Persistence 职责边界 / dependency direction | PASS |
| migration 内容冻结 | PASS |
| architecture/frontend/workspace Rust/Windows Automated | PASS |
| 空库 migration 0001–0046 | PASS (`31729577225`) |
| health/stats PostgreSQL integration | PASS (`31729577225`) |
| audit rollback + commit integration | PASS (`31729577225`) |
| destructive reset（临时 test DB） | PASS (`31728096953`) |
| R4 stage | **DONE** |

## 17. 下一阶段唯一 READY 任务

R5-01 `Competitions Repository` 为唯一 `READY`；R5-02~R5-06 `BLOCKED`。R5 stage 索引已创建，但没有创建、迁移或修改任何 R5 生产实现。R5-01 必须从最终通过 canonical CI 的 R4 closeout HEAD 独立建立分支。

## 18. 订正记录

- broad `31728096953` 首次实跑此前 ignored 的 18 个数据库测试，14/18；没有把跨业务 18/18 错误替代 R4 stage matrix。scoped `31729361081` 的 audit probe 仅因 runner session-local trigger 设计失败；final scoped `31729577225` 真实验证 R4 matrix 并通过，因此 R4 stage 从 `VERIFYING` 订正为 `DONE`。
- docs-only closeout 第二次 run `31727494199` 已成功生成并发布四份收口文档到 stage commit `837d364cbddb174f794529efb4b7be76fa673f5e`；随后仅 transient helper 自清理因 runner 内临时改写 helper 导致 `git rm` 拒绝而失败，未影响 canonical stage。assistant-created helper/workflow 已随后通过 connector 提交 `dc992b7630f1559c1aaa0e10fd2a30626450d7da` / `acc9884277355410a1f960ee91c0bcc2e8f4b634` 从 helper 分支删除；canonical stage 从未包含这些 transient 文件。
- docs-only closeout 首次 run `31727111563` 在发布前 fail-fast：scope 校验仅读取 tracked `git diff`，遗漏新建且仍 untracked 的 `R04-stage-completion.md`；stage 发布和 transient cleanup 步骤均被跳过，canonical stage 未发生变化。恢复仅将 scope 校验改为 tracked diff 与 untracked 文件的精确并集，最终允许集合仍严格为四份收口文档。
- R4-04 节点在 hard gate、clean PR CI、squash merge 与 post-merge stage CI 全部成功后关闭为 `DONE`。
- 阶段收口时重新按 R4 任务书检查“阶段级验证矩阵”，确认 18 个 PostgreSQL integration tests / 空库 migration / health-stats / audit integration 仍未执行；因此没有沿用“节点全部 DONE 即阶段 DONE”的简化判断，而是将 R4 stage 保持 `VERIFYING`。该订正只影响阶段状态和文档，不修改生产源码、接口、数据或验证门禁。
