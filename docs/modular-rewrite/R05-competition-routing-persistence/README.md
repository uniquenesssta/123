# R05 Competition / Routing Persistence：执行记录索引

## 阶段状态

`IN_PROGRESS`

R4 Persistence 基础设施已完成并通过阶段出口。R5 仅按 Competition、Season、Stage、Round、Rule Package、Binding、Route Resolution 与 Model Run Identity 拆分 PostgreSQL Adapter；不修改模型路由算法、前端赛事页面或历史 migration。

## 前置基线

- R4 `DONE`；完成记录：[`../R04-persistence-foundation/R04-stage-completion.md`](../R04-persistence-foundation/R04-stage-completion.md)。
- R4 最终 closeout HEAD：`615dc952491d5e0e21d4979292cbf5170eedece6`。
- 该 HEAD 的 canonical Public Platform CI run `31768264193` / Windows automated delivery gate job `94668577704` 为 `SUCCESS`。
- R5 stage 分支 `rewrite/r5-competition-routing-persistence` 当前已完成 R5-02 closeout；R5-03 从最终验证基线 `bcd17c78dde89c999d7666fa41e7267f609c6a33` 精确建立。
- R5-02 最终 closeout canonical Public Platform CI run `31798978218` / job `94762276340` 为 `SUCCESS`。
- R3 Competition / Rules Ports 已冻结；0001–0046 migration 继续冻结。

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R5-01 | Competitions Repository | DONE |
| R5-02 | Seasons / Stages / Rounds | DONE |
| R5-03 | Rule Packages | DONE |
| R5-04 | Competition Bindings | READY |
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

## R5-03 当前事实

- 详细记录：[`R05-03-rule-packages.md`](R05-03-rule-packages.md)。
- Rule Package 的 register/list、source document upsert 与 Row mapping 已从旧 `crates/persistence-postgres/src/routing.rs` 拆入 `adapters/rules/packages/`；transaction、package insert、existing conflict read、profile attach、source upsert、typed Row 与 Domain Mapper 均为独立职责文件。
- 旧 `routing.rs` 已删除全部 R5-03 owner，不保留转发壳；R5-04 Binding、R5-05 Route Resolution 与 R5-06 Model Run Identity 仍未提前迁移。共享 `register_model_in_tx` 仅提升为 crate 内可见供 Rule Package 事务复用。
- old owner PostgreSQL 16 contract run `31810196287` / job `94798701210` 为 `SUCCESS`；new owner 同一 contract run `31811073556` / job `94801559817` 亦为 `SUCCESS`。
- owner-switch 首次 run `31810593242` 因 source helper 可见性 E0364/E0603 停止且未产生生产切换提交；repair run `31810761979` 的代码修正成功但 Action 写 workflow 被 GitHub App 权限阻止；V2 run `31810913317` / job `94801046458` 完成真实编译、专项 verifier 与生产 owner 切换并 `SUCCESS`。
- official inventory run `31811229904` / job `94802081839` 为 `SUCCESS`；第一次 hard gate `31811298187` 在 architecture/model 通过后因新增 contract rustfmt 差异停止，后续正确跳过；format + inventory run `31811465209` / job `94802847771` 为 `SUCCESS`。
- 第二轮 hard gate run `31811535324` / job `94803075524` 为 `SUCCESS`：R5 ownership/full architecture、模型保护、rustfmt、Persistence/Application check、Persistence tests、Application tests 与同一 Rule Package PostgreSQL 16 contract 全部通过。
- PR #28 clean CI run `31812316772` / job `94805647551` 为 `SUCCESS`，artifact `9224448312` SHA-256 `00cf61b683b7780a354ad5e35c59438d6d26ce1a6d0c17dd86edbaaa9e9ae127`；固定 head `d087108d5cc722cc9ebecd788de6c07c1ae5dc3d` 已 squash merge 为 `3d609ace7cbe1db3a3caec18a6477fb41153cc88`。
- merged stage CI run `31824523536` / job `94845378301` 为 `SUCCESS`，artifact `9229038667` SHA-256 `9c07c35abcd8b229b8044b0373f7ce3aecd6a03d58b1431c4f738e22131c7262`；R5-03 正式关闭为 `DONE`，R5-04 开放为 `READY`。

## 兼容与限制

- Application Port、Tauri 命令/DTO、Schema、0001–0046 migration、配置、错误/日志语义、前端行为、路由算法、model identity、Cargo manifests/Cargo.lock、生产依赖和模型保护资产均未改变。
- R5-03 未执行 destructive database reset，也未触碰用户数据库；专用 Rule Package contract 使用临时 PostgreSQL 16 测试数据库。
- 既有 `postgres_integration.rs` 18 个 ignored broad PostgreSQL tests 未在 R5-03 执行。
- R5-03 已正式关闭为 `DONE`，R5-04 已开放为 `READY`；本 closeout 仅更新文档，closeout HEAD 仍需通过 canonical Public Platform CI 后才作为 R5-04 起始基线。
