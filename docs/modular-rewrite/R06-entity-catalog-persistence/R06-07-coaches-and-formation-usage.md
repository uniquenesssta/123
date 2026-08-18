# R6-07 Coaches 与 Formation Usage

## 状态

`DONE`

## 基线与范围

- 唯一阶段分支：`rewrite/r6-entity-catalog-persistence`。
- 节点起点：`7bf9b90bc235bdb397d003bdd6e61458e2e425c9`。
- 本节点仅重写 Coaches 与 Formation Directory / Usage / Resolution persistence；R6-08 Entity Matching / References 与 R6-09 Archive / Delete / Force Delete 未提前迁移。

## 实现结果

- Coach Directory、Detail、Names、Team Periods、Validation/Existence/Normalization、typed Row / Domain Mapping 已收敛到 `crates/persistence-postgres/src/adapters/catalog/coaches/`。
- Formation Directory、Usage、Resolution 已收敛到 `crates/persistence-postgres/src/adapters/catalog/formations/`。
- Formation resolution SQL 读取集中到 `resolution/read.rs`；Usage 保存准备、窗口读取和 append-only 写入分别集中到 `usage/preparation.rs`、`usage/window/read.rs`、`usage/write.rs`，协调器保持 SQL-free。
- 旧 `crates/persistence-postgres/src/formation_catalog.rs` 已删除；`entity_catalog.rs` 已移除 Coach owner，但继续保留 R6-08/R6-09 matching/reference/archive/delete 职责。
- 历史 Global Name Search、Entity Relationships 与 Formation Usage verifier 只跟随到新的权威 owner，原断言、数量阈值和失败语义未降低。
- 无新增生产依赖。

## 契约与兼容性

- `CoachCatalogPort`、`FormationPort` 公共方法签名和调用语义保持不变。
- Domain DTO、Schema、0001–0046 migrations、配置、默认值、错误类型/文案、日志等级、安全策略、默认战术角色/位置映射和用户可观察行为保持不变。
- Formation Usage 继续 append-only；smoothing、UNKNOWN fallback、actual/confirmed 优先级与当前教练角色优先级保持原语义。
- 历史 P4/运行引用未删除、改绑或级联破坏。

## 验证事实

- Windows 本地 Minimum Gate 已通过：official Domain inventory、完整 architecture、Persistence `cargo check`、95/95 unit tests、R6-07 contract 编译、Application check、rustfmt。
- Windows 本地 Stage Regression 已通过：`npm run verify:frontend`、TypeScript、Vite build、`cargo fmt --all -- --check`、workspace Clippy `-D warnings`、`cargo test --locked --workspace`、`git diff --check`。
- R6-07 hard-gate run `32125684434` / job `95675605447` 中，ownership/compatibility、保护资产/database static baseline、workspace Clippy、workspace tests、R6-03～R6-07 PostgreSQL contracts 与 Formation Usage PostgreSQL integration 全部通过；该 run 仅在额外的 full frozen PostgreSQL baseline 停止。
- R6-07 final PostgreSQL gate run `32128507369` / job `95684279881` 为 `SUCCESS`：R6-07 architecture/frozen boundaries、Persistence/Application compile、R6-03～R6-07 PostgreSQL contracts 与 Formation Usage PostgreSQL integration 全部真实通过。
- 额外 full frozen PostgreSQL baseline 诊断：当前 R6-07 tree run `32127380673` 与节点起点 `7bf9b90bc235bdb397d003bdd6e61458e2e425c9` reference run `32127587771` 均为相同的 14/18 PASS、4 FAIL。失败项均为：
  - `match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent`
  - `match_scope_inference_and_lineup_pair_transaction_are_atomic`
  - `p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable`
  - `structured_match_events_are_queryable_and_revision_aware`
- 上述 4 项在 R6-07 开始前已以相同错误失败，确认不是 R6-07 引入；本节点未删除、跳过、放宽这些测试，也未越界修改对应 Lineup/P4/Review owner。

## 未执行与剩余风险

- 用户现有 PostgreSQL 数据库真实数据 sample/write 验收未执行，继续保留到最终统一验收。
- Windows Full 人工交互验收未执行，继续保留到最终统一验收。
- 上述 4 个历史 PostgreSQL 集成测试既有失败仍存在，后续由对应职责节点处理。

## 清理与回退

- 临时 payload/bootstrap、hard-gate、final-PG、database-diagnostic workflow/result 均已清理；最终 closeout workflow 在本次收口提交中自删除。
- 回退点为 `7bf9b90bc235bdb397d003bdd6e61458e2e425c9`；不得通过恢复双 owner 或旧 `formation_catalog.rs` 进行手工回退。
