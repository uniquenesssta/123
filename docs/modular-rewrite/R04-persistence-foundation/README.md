# R04 Persistence 基础设施重写：执行记录索引

## 阶段状态

`IN_PROGRESS`

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
| R4-03 | 通用 Row 映射基础规范 | VERIFYING | [`R04-03-row-映射基础规范.md`](./R04-03-row-映射基础规范.md) |
| R4-04 | Application Port Adapter 注册 | BLOCKED | — |

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

## R4-03 实施中

- 从 R4-02 正式收口 HEAD `3c147376cf81394984cc20850a58e866eef4280b` 独立建立 `agent/r4-03-row-mapping`。
- run `31665585062` 因 verifier 转义错误与共享调用面漏扫停止；run `31665943169` 因临时 workflow YAML 缩进错误在调度前停止；两次均未提交生产源码。 run `31666083511` 随后通过 R4-03 最小门禁与完整 architecture，但因临时 runner 未先执行既有 `npm run setup`，frontend 在缺少已声明的 `typescript` 开发依赖时停止；workspace Clippy/tests 未执行且仍未提交生产源码。run `31666505364` 已通过全部质量、冻结范围、文档与 transient cleanup 门禁，最终仅因 npm cache 临时落在仓库根而被 clean-worktree 发布门禁拒绝，仍未提交生产源码。run `31667443481` 再次通过全部质量、范围与文档门禁，但 cleanup 在精确暂存前检查 untracked，误将本任务应新增文件视为非法项并停止，仍未提交生产源码。run `31668274055` 通过全部质量、范围、文档、精确暂存与 untracked 清理门禁，最终仅因 Git 默认 quotePath 将中文记录文件名八进制转义而误判 staged allowlist，仍未提交生产源码。
- 恢复 run `31674700550` 已通过全部最小门禁与阶段回归。
- R4-03 当前 `VERIFYING`；clean Public Platform CI 与 PR 合并完成前不得标记 `DONE`，R4-04 继续 `BLOCKED`。
