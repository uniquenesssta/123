# R05 Competition / Routing Persistence：执行记录索引

## 阶段状态

`IN_PROGRESS`

R4 Persistence 基础设施已完成并通过阶段出口。R5 仅按 Competition、Season、Stage、Round、Rule Package、Binding、Route Resolution 与 Model Run Identity 拆分 PostgreSQL Adapter；不修改模型路由算法、前端赛事页面或历史 migration。

## 前置基线

- R4 `DONE`；完成记录：[`../R04-persistence-foundation/R04-stage-completion.md`](../R04-persistence-foundation/R04-stage-completion.md)。
- R4 最终 closeout HEAD：`615dc952491d5e0e21d4979292cbf5170eedece6`。
- 该 HEAD 的 canonical Public Platform CI run `31768264193` / Windows automated delivery gate job `94668577704` 为 `SUCCESS`。
- R5 stage 分支 `rewrite/r5-competition-routing-persistence` 当前已完成 R5-01 closeout，R5-02 从 `c8f3ba0f35ccec1f2795328fe887c622a5b8a0f6` 精确建立。
- R5-01 closeout HEAD canonical Public Platform CI run `31779160119` 为 `SUCCESS`。
- R3 Competition / Rules Ports 已冻结；0001–0046 migration 继续冻结。

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R5-01 | Competitions Repository | DONE |
| R5-02 | Seasons / Stages / Rounds | VERIFYING |
| R5-03 | Rule Packages | BLOCKED |
| R5-04 | Competition Bindings | BLOCKED |
| R5-05 | Route Resolution Reads | BLOCKED |
| R5-06 | Model Run Identity Reads | BLOCKED |

## R5-01 当前事实

- 详细记录：[`R05-01-competitions-repository.md`](R05-01-competitions-repository.md)。
- Competition 的 `create/read/list/delete` 已从旧 `crates/persistence-postgres/src/competitions.rs` 收敛到 `adapters/competition/directory/` 与 `detail/`；typed Row 与 Domain Mapper 分离，多表 soft delete 使用具名 transaction 目录。
- `competitions.rs` 在 R5-01 结束时仅保留 R5-02/R5-05 尚未迁移的 Season/Stage/Round 与 scope context 职责；R5-02 当前已进一步移出 hierarchy 职责。
- 新增 PostgreSQL contract 先在旧实现基线上运行：run `31772436658` / job `94680947724`，`SUCCESS`；切换新 owner 后在 PostgreSQL 16 再次 1/1 `PASS`。
- 最终 Ubuntu stage diagnostic run `31773549494` / job `94684193535` 中，R5-01/R4 专项、数据库与保护资产、Rust minimum、Persistence 80/80、Application 33/33、PostgreSQL contract 1/1、完整 architecture 均 `SUCCESS`；该 run 的总体失败仅来自非目标 Ubuntu Chromium 截图环境，未将其记为 frontend 通过。
- Windows frontend 在 run `31773777077` / job `94684879652` 完整 `SUCCESS`。
- Windows workspace Rust 在 run `31773900831` / job `94685243308` 完成 `cargo clippy --locked --workspace --all-targets -- -D warnings` 与 `cargo test --locked --workspace`，均 `SUCCESS`。
- R5-01 PR #26 clean CI 与 merged stage CI 均已通过；最终 closeout HEAD `c8f3ba0f35ccec1f2795328fe887c622a5b8a0f6` 的 run `31779160119` 亦为 `SUCCESS`。

## R5-02 当前事实

- 详细记录：[`R05-02-seasons-stages-and-rounds.md`](R05-02-seasons-stages-and-rounds.md)。
- Season / Stage / Round 已分别拆入 `adapters/competition/hierarchy/seasons/`、`stages/`、`rounds/`；每个模块分别拥有 create/read/list、typed Row 和 Domain Mapper。
- 旧 `crates/persistence-postgres/src/competitions.rs` 已删除全部 R5-02 CRUD/read/dynamic PgRow mapper，只保留 R5-05 `resolve_competition_context` 与 scope 校验；没有保留 R5-02 转发壳。
- 旧 owner PostgreSQL contract 在生产切换前 run `31793408123` 为 `SUCCESS`；首次 baseline `31793270046` 仅因新测试引用不存在的 enum variant 而编译失败，当时生产源码未修改。
- 第一轮 hard gate `31793915203` 在 architecture / protected boundary 成功后因 canonical rustfmt 停止；rustfmt fixer run `31793982624` 成功且仅修改格式。
- 第二轮 hard gate `31794033828` 正确发现 rustfmt 后 domain inventory usage digest 漂移并停止；官方 generator refresh run `31794157058` 为 `SUCCESS`。
- 第三轮 hard gate run `31794195852` 正在执行；未完成前 R5-02 保持 `VERIFYING`，R5-03 保持 `BLOCKED`。

## 兼容与限制

- Application Port、Tauri 命令/DTO、Schema、0001–0046 migration、配置、错误/日志语义、前端行为、路由算法、model identity、Cargo manifests/Cargo.lock、生产依赖和模型保护资产均未改变。
- R5-02 未执行 destructive database reset，也未触碰用户数据库；专用 hierarchy contract 使用临时 PostgreSQL 16 测试数据库。
- 既有 `postgres_integration.rs` 18 个 ignored broad PostgreSQL tests 未在 R5-02 执行；clean PR CI、workspace Clippy/tests、Windows Automated、固定 HEAD 合并和 merged stage CI 尚待完成。
- R5-02 只有在全部最终门禁成功、临时 workflow 清理、README/节点记录收口后才可 `DONE`；届时才开放 R5-03 `READY`。
