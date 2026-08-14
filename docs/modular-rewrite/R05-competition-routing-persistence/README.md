# R05 Competition / Routing Persistence：执行记录索引

## 阶段状态

`IN_PROGRESS`

R4 Persistence 基础设施已完成并通过阶段出口。R5 仅按 Competition、Season、Stage、Round、Rule Package、Binding、Route Resolution 与 Model Run Identity 拆分 PostgreSQL Adapter；不修改模型路由算法、前端赛事页面或历史 migration。

## 前置基线

- R4 `DONE`；完成记录：[`../R04-persistence-foundation/R04-stage-completion.md`](../R04-persistence-foundation/R04-stage-completion.md)。
- R4 最终 closeout HEAD：`615dc952491d5e0e21d4979292cbf5170eedece6`。
- 该 HEAD 的 canonical Public Platform CI run `31768264193` / Windows automated delivery gate job `94668577704` 为 `SUCCESS`。
- R5 stage 分支 `rewrite/r5-competition-routing-persistence` 与 R5-01 实施分支 `agent/r5-01-competitions-repository` 均从上述最终 R4 HEAD 精确建立。
- R3 Competition / Rules Ports 已冻结；0001–0046 migration 继续冻结。

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R5-01 | Competitions Repository | DONE |
| R5-02 | Seasons / Stages / Rounds | READY |
| R5-03 | Rule Packages | BLOCKED |
| R5-04 | Competition Bindings | BLOCKED |
| R5-05 | Route Resolution Reads | BLOCKED |
| R5-06 | Model Run Identity Reads | BLOCKED |

## R5-01 当前事实

- 详细记录：[`R05-01-competitions-repository.md`](R05-01-competitions-repository.md)。
- Competition 的 `create/read/list/delete` 已从旧 `crates/persistence-postgres/src/competitions.rs` 收敛到 `adapters/competition/directory/` 与 `detail/`；typed Row 与 Domain Mapper 分离，多表 soft delete 使用具名 transaction 目录。
- `competitions.rs` 仅保留 R5-02/R5-05 尚未迁移的 Season/Stage/Round 与 scope context 职责；没有保留 R5-01 转发壳，也没有提前实施后续节点。
- 新增 PostgreSQL contract 先在旧实现基线上运行：run `31772436658` / job `94680947724`，`SUCCESS`；切换新 owner 后在 PostgreSQL 16 再次 1/1 `PASS`。
- 最终 Ubuntu stage diagnostic run `31773549494` / job `94684193535` 中，R5-01/R4 专项、数据库与保护资产、Rust minimum、Persistence 80/80、Application 33/33、PostgreSQL contract 1/1、完整 architecture 均 `SUCCESS`；该 run 的总体失败仅来自非目标 Ubuntu Chromium 截图环境，未将其记为 frontend 通过。
- Windows frontend 在 run `31773777077` / job `94684879652` 完整 `SUCCESS`。
- Windows workspace Rust 在 run `31773900831` / job `94685243308` 完成 `cargo clippy --locked --workspace --all-targets -- -D warnings` 与 `cargo test --locked --workspace`，均 `SUCCESS`。
- 一次性 `.github/workflows/r5-01-contract-baseline.yml` 已删除，不进入最终净 diff。

## 兼容与限制

- Application Port、Tauri 命令/DTO、Schema、0001–0046 migration、配置、错误/日志语义、前端行为、路由算法、model identity、Cargo manifests/Cargo.lock、生产依赖和模型保护资产均未改变。
- 既有 `postgres_integration.rs` 18 个 ignored broad PostgreSQL tests 未在 R5-01 执行；未执行 destructive database reset；未触碰用户数据库。
- PR #26 clean Public Platform CI run `31775501711` / job `94689993582` 为 `SUCCESS`；固定 head `999949da6b3d5aeff4734b68779d3b15a890abe1` 已 squash merge 为 `2b4668c2c0da45c9f62c1d40c4765cd730653b33`。
- merged stage Public Platform CI run `31777130456` / job `94694820128` 为 `SUCCESS`；artifact `9210904946`，SHA-256 `1303176d639550a832327c6e3e6c564f843e7eed10b673144ecd6413b7b01d31`。
- R5-01 正式 `DONE`；R5-02 已开放为 `READY`，R5 stage 继续 `IN_PROGRESS`。最终文档 closeout HEAD 仍需通过 canonical Public Platform CI 后作为下一节点基线。
