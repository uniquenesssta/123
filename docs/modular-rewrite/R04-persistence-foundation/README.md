# R04 Persistence 基础设施重写：执行记录索引

## 阶段状态

`DONE`

R3 Application Services 已完成并关闭。R4 只重写 `crates/persistence-postgres/src/` 的 PostgreSQL Adapter 基础设施、错误、审计和基础映射边界；不修改具体业务 SQL，不修改历史迁移，不修改 Tauri/前端产品行为或模型保护资产。

## 基线

- R4 阶段分支：`rewrite/r4-persistence-foundation`
- R4 起点：`f2e4841aac873f6a4812801e6be2a6524cd680c1`
- R3 完成记录：[`../R03-application-services/R03-stage-completion.md`](../R03-application-services/R03-stage-completion.md)
- R3 最终 canonical Public Platform CI：run `31608755744`，`SUCCESS`
- 目标平台：Windows

## 任务状态

| 任务 | 范围 | 状态 | 记录 |
|---|---|---|---|
| R4-01 | Store / Error / Pool / migration / health / statistics | DONE | [`R04-01-store-error-and-pool.md`](./R04-01-store-error-and-pool.md) |
| R4-02 | Audit 基础设施 | DONE | [`R04-02-audit-基础设施.md`](./R04-02-audit-基础设施.md) |
| R4-03 | 通用 Row 映射基础规范 | DONE | [`R04-03-row-映射基础规范.md`](./R04-03-row-映射基础规范.md) |
| R4-04 | Application Port Adapter 注册 | DONE | [`R04-04-port-adapter-注册.md`](./R04-04-port-adapter-注册.md) |

## R4-01 进入条件

- R3 Ports 已冻结并完成阶段收口。
- R3 最终阶段分支 HEAD `f2e4841aac873f6a4812801e6be2a6524cd680c1` 的 canonical CI 已通过。
- 历史迁移 0001–0046 保持冻结；R4-01 不修改任何 migration SQL。

R4-01 完成前保持 `READY/VERIFYING`，只有目标职责切换为唯一 owner、旧职责实现清理、最小验证与阶段回归实际通过、根 `README.md` 与节点记录同步后才能标记为 `DONE`。

## R4-01 收口

- PR #22 已合并到 `rewrite/r4-persistence-foundation`，merge commit `ed99ce3bc21da76a55de526a7f912fa363bb84a5`。
- 最终 clean Public Platform CI run `31615930113` / job `94178594481`：`SUCCESS`；artifact `9150134498`，13,909,168 bytes，SHA-256 `6da5f1750feef5f7fc00233165dea7f7563fefa96d08f8e515a22e16fa4dd5e1`。
- R4-01 正式关闭为 `DONE`；R4-02 开放为 `READY`，R4-03/R4-04 继续 `BLOCKED`。
- R4-02 必须从本阶段分支当前收口基线独立开始；本收口未包含任何 R4-02 生产源码改动。

## R4-02 收口

- PR #23 已由 Draft 转 Ready，并按固定 HEAD `6598065e6edf671e8806fc77901d083bad910542` 合并到 `rewrite/r4-persistence-foundation`；merge commit `bc4154044f194e5b1505b4ebb308ba51d6208663`。
- 最终 clean Public Platform CI run `31630618699` / job `94228122818`：`SUCCESS`；artifact `9155761101`，13,909,390 bytes，SHA-256 `82dbb88b6dd974ad43c2d86bfb291aca8831faa9d956212bdc04de7110d87dee`。
- strict hard gate run `31629430245` 已通过 Audit/Persistence/数据库冻结契约、Domain inventory、architecture/frontend、rustfmt、Persistence check/tests 与 workspace Clippy/tests。
- 18 个要求专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试仍未执行；未执行 destructive database reset。
- R4-02 正式关闭为 `DONE`；R4-03 开放为 `READY`，R4-04 继续 `BLOCKED`。
- R4-03 必须从本阶段分支当前收口基线独立开始；本收口未包含任何 R4-03/R4-04 生产源码改动。

## R4-03 收口

- 从 R4-02 正式收口 HEAD `3c147376cf81394984cc20850a58e866eef4280b` 独立建立 `agent/r4-03-row-mapping`；最终实现提交为 `202648df6aa1f9a14eb03bdcabcbd5ee0a271e56`。
- strict hard gate run `31674700550` 已通过 R4-03 mapping 专项、R4-01/R4-02 Persistence/Audit 回归、数据库冻结与保护资产、rustfmt、Persistence check/tests、完整 architecture/frontend、workspace Clippy `-D warnings` 与 workspace tests；最终实现树严格为 15 个目标文件，临时 workflow/helper 与额外 untracked 均为 0。
- PR #24 clean Public Platform CI run `31675727990` / Windows automated delivery job `94369762967`：`SUCCESS`；PR 按固定 HEAD `202648df6aa1f9a14eb03bdcabcbd5ee0a271e56` 合并到 `rewrite/r4-persistence-foundation`，merge commit `0e5a68e09c6c06204b926f4d30c45262740d983b`。
- 合并后 stage Public Platform CI run `31677600876` / job `94375513281`：`SUCCESS`；artifact `9172855279`（`windows-automated-delivery-evidence-0e5a68e09c6c06204b926f4d30c45262740d983b`）大小 `13909093` 字节，SHA-256 `31719ff04f00eb944c84fcd37dbbda3fa6252f7d55bc42a1a8af3d903cad3544`。
- formal-closeout preparation runs `31679755024` 与 `31679826851` 因旧 workflow 的内嵌 Python 多行文本破坏 YAML block 缩进而在调度前失败（0 job）；run `31679847327` 已进入 helper，但因状态校验误把历史兼容段的第二个 `VERIFYING` 也计入而 fail-fast；run `31679942355` 已生成目标文档内容，但 `git diff --check` 检出 stage README 尾部新增空白行后停止。四次均未产生 closeout commit、未修改生产源码；恢复 helper 改为精确状态 marker 并规范单个 EOF 换行，最终提交前 workflow/helper 均自删除。
- formal closeout workflow run `31680032287` 成功生成并推送文档收口提交 `56f22e0d2afab301668e6cfb8b1b447d59b58150`；该提交相对生产 merge commit 的净变化严格只有根 `README.md`、本阶段索引和 R4-03 节点记录三份文档，临时 `.github` workflow/helper 为零差异。
- 18 个要求专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试仍未执行；未执行 destructive database reset。
- R4-03 正式关闭为 `DONE`；R4-04 开放为 `READY`。本收口未包含任何 R4-04 生产源码改动。

## R4-04 收口

- 从 R4-03 最终 canonical HEAD `b508d1ff7808b8735694cf1e57a4d603f2e973e3` 独立实施 R4-04；scope audit run `31686092182` 确认旧入口为 `ActiveDatabase` + `transition_store`，40 个 DB-backed Application Ports 由该 wrapper 转发到 PostgresStore。
- Windows 2025 / Rust 1.88.0 / Node 22 hard gate run `31712193150` 已通过 R4-04 专项、R4-01/R4-02/R4-03 回归、数据库静态冻结、architecture/frontend、Persistence/Application check/tests、workspace Clippy `-D warnings` 与 workspace tests；Persistence 80/80、Application 33/33 tests 通过，18 个 PostgreSQL integration tests 保持 ignored。
- PR #25 clean Public Platform CI run `31717901248` / Windows automated delivery job `94507174688`：`SUCCESS`；artifact `9188959652`，13,894,348 bytes，SHA-256 `477b32b66fb06ee9bfa04ead7778639f1d5658f9478a641c706416c49b70d687`。最终 PR HEAD `b808ff3cc7a43f3c68b0f024228adefb20b916cb` 与已验证 implementation tree `a7f1dc852bc486dbf0b1ca4dd7db5d262950d7c8` 文件内容零差异，assistant-created `.noop` / marker 未进入最终 diff。
- PR #25 按固定 HEAD `b808ff3cc7a43f3c68b0f024228adefb20b916cb` 以 squash merge 合并到 `rewrite/r4-persistence-foundation`；stage merge commit `b97587c9d20165018f80040dc2a2c098dbbec177`。
- 合并后 canonical stage Public Platform CI run `31724131556` / job `94528167665`：`SUCCESS`；artifact `9191415520`（`windows-automated-delivery-evidence-b97587c9d20165018f80040dc2a2c098dbbec177`）大小 13,895,272 bytes，SHA-256 `3ccb37d8eab17c0e589354c10c3423579397629f9f76481b267acd0749db38cf`。
- R4-04 正式关闭为 `DONE`。公共 Application Ports、Tauri command/DTO、SQL、0001–0046 migrations、配置、错误语义、模型保护资产和生产依赖均未改变。

## R4 阶段出口状态

- 四个 Atomic Task 均已 `DONE`，阶段完成记录见 [`R04-stage-completion.md`](./R04-stage-completion.md)。
'
'- broad PostgreSQL diagnostic `31728096953` 在一次性 `postgres:16` / `football_r4_test` 上执行全部 18 个 ignored integration tests：14/18 PASS，destructive reset PASS。4 个失败未删除、跳过或放宽：3 个为当前严格业务契约下的既有过期夹具，1 个为具体 P4 业务持久化 timestamp 精度问题；相关业务路径未由 R4 修改，P4 具体业务持久化属于 R4 排除范围。
'
'- scoped run `31729361081` 已通过空库 migration、health、stats，但 runner 的 connection-local audit trigger 参数设计错误导致 probe 失败；未改生产源码。修正 runner-only trigger 后，final run `31729577225` / job `94546316946` `SUCCESS`：全新临时库真实完成 46 条 migration、`health()`、`stats()`、audit 失败事务整体回滚及成功事务业务 row + audit row 同时提交，1/1 PASS。
'
'- architecture dependency 与 migration freeze 已由 hard gate / canonical Windows CI 通过；R4 阶段出口矩阵全部 PASS，R4 正式 `DONE`。
'
'- 临时 PostgreSQL service、runner-only Rust test 与 transient workflows 均已清理，未进入 canonical stage，也未连接或重置用户数据库。
'
'- R5-01 `Competitions Repository` 为唯一 `READY`；R5-02~R5-06 `BLOCKED`。本收口未包含任何 R5 生产源码修改。
