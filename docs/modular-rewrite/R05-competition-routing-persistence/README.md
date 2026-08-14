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
| R5-02 | Seasons / Stages / Rounds | DONE |
| R5-03 | Rule Packages | READY |
| R5-04 | Competition Bindings | BLOCKED |
| R5-05 | Route Resolution Reads | BLOCKED |
| R5-06 | Model Run Identity Reads | BLOCKED |

## R5-01 当前事实

- 详细记录：[`R05-01-competitions-repository.md`](R05-01-competitions-repository.md)。
- Competition 的 `create/read/list/delete` 已从旧 `crates/persistence-postgres/src/competitions.rs` 收敛到 `adapters/competition/directory/` 与 `detail/`；typed Row 与 Domain Mapper 分离，多表 soft delete 使用具名 transaction 目录。
- R5-01 PR #26、merged stage CI 和最终 closeout HEAD `c8f3ba0f35ccec1f2795328fe887c622a5b8a0f6` 的 canonical run `31779160119` 均已通过。

## R5-02 当前事实

- 详细记录：[`R05-02-seasons-stages-and-rounds.md`](R05-02-seasons-stages-and-rounds.md)。
- Season / Stage / Round 已分别拆入 `adapters/competition/hierarchy/seasons/`、`stages/`、`rounds/`；每个模块分别拥有 create/read/list、typed Row 和 Domain Mapper。
- 旧 `crates/persistence-postgres/src/competitions.rs` 已删除全部 R5-02 CRUD/read/dynamic PgRow mapper，只保留 R5-05 `resolve_competition_context` 与 scope 校验；没有保留 R5-02 转发壳。
- 旧 owner PostgreSQL contract 在生产切换前 run `31793408123` 为 `SUCCESS`；首次 baseline `31793270046` 仅因新测试错误引用不存在的 enum variant 而编译失败，当时生产源码未修改。
- 第一轮 hard gate `31793915203` 因 canonical rustfmt 停止；第二轮 `31794033828` 正确发现 rustfmt 后 domain inventory digest 漂移；第三轮 `31794195852` 在 architecture/model/rustfmt 通过后暴露内部 re-export visibility E0364/E0365。上述问题均按硬门禁修复，没有弱化检查。
- official inventory generator 最终 refresh run `31794365962` / job `94748073290` 为 `SUCCESS`。
- 第四轮 hard gate run `31794402789` / job `94748190282` 为 `SUCCESS`：R5-01/R5-02 ownership、完整 architecture、模型保护、rustfmt、Persistence/Application check、Persistence tests、Application tests、同一份 PostgreSQL 16 hierarchy contract 全部通过。
- PR #27 clean CI run `31794818136` / job `94749470982` 为 `SUCCESS`，固定 clean head `89f164821e8f8157ab8a4804cf1d026a40ab8932` 已 squash merge 为 `baf307fcb733659385f83f187ee343184946ee9d`；merged stage CI run `31796688330` / job `94755204665` 亦为 `SUCCESS`。PR artifact `9217614243` SHA-256 `e1adc3016f71f1c563b9fd0d29721a9f04452cccfe5661f6bb5be52924f7601a`，stage artifact `9218314804` SHA-256 `80e836c4012524ac66a21b085b86bebcd4f3fdf4661594d636050fb753f98d65`。

## 兼容与限制

- Application Port、Tauri 命令/DTO、Schema、0001–0046 migration、配置、错误/日志语义、前端行为、路由算法、model identity、Cargo manifests/Cargo.lock、生产依赖和模型保护资产均未改变。
- R5-02 未执行 destructive database reset，也未触碰用户数据库；专用 hierarchy contract 使用临时 PostgreSQL 16 测试数据库。
- 既有 `postgres_integration.rs` 18 个 ignored broad PostgreSQL tests 未在 R5-02 执行；未执行 destructive database reset，未触碰用户数据库。clean PR 与 merged stage canonical Windows Automated 均已通过。
- R5-02 已正式关闭为 `DONE`，R5-03 已开放为 `READY`；本次 closeout 文档 HEAD 必须先通过 canonical Public Platform CI，才可作为 R5-03 的起始基线。
