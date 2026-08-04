# R00 基线冻结与可重复验收：执行记录索引

## 阶段范围

冻结模块化重写开始前的远端提交、分支、保护资产、公共命令、数据库、前后端与 Windows 验收基线。R00 不实施业务重写，不移动源码，不修改模型保护区。

## 当前基线

- 基线分支：`main`
- 起始基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- R0-02 起始提交：`beb00b6f723d75703e9235967c4f49f985d41c4e`
- R0-03 起始提交：`83a7d0bddd83894b4f551725c40b50705470e731`
- R0-04 起始提交：`8cddf452e01770a042b671ed8c11bc29afd2f1b1`
- 实施分支：`new-A`
- 已完成节点：`R0-01`、`R0-02`、`R0-03`
- 验证中节点：`R0-04`

## 保护资产

- `architecture/protected-assets.json` 已冻结 18 个公开模型边界及校验文件。
- 聚合 SHA-256：`d2263a5ff09c8cf633a42b7bb35fffe3d42fb18648db4d12691817f51015c85c`。
- `crates/model-api`、`crates/model-stub`、`crates/application/src/model_shell` 和 `contracts/model-*.json` 使用精确文件集合门禁。
- 真实 P4/P7 源码、参数、Profile、专用 Schema、fixture、Golden Master 和私有研究资产继续禁止进入公开仓库。

## 命令契约

- `architecture/command-contract.json` 已冻结 171 个公共命令、注册顺序和 15 个 Rust 命令模块。
- 命令集合 SHA-256：`dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669`。
- 命令定义映射 SHA-256：`eb908c4ae31ec8d49e65a1b336c1105cb550e755fcdbae40f4943f99433cc9f5`。
- 前端参数和返回签名位于 `src/api/client.ts`，共享前端 DTO 位于 `src/types.ts`；Rust 参数和返回签名位于各命令定义文件，注册入口位于 `src-tauri/src/lib.rs`。

## 数据库基线

- `architecture/database-baseline.json` 已冻结 `0001`–`0046` 共 46 个连续迁移，当前没有 `0047+` 迁移。
- 迁移聚合 SHA-256：`d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`。
- `scripts/verify_database_baseline.mjs` 校验迁移集合、内容指纹、SQLx 迁移入口、18 个 PostgreSQL 集成测试契约和关键不可变触发器。
- `scripts/run_database_baseline.mjs` 只允许数据库名称包含 `test`，并在静态门禁通过后执行全部被忽略的 PostgreSQL 集成测试。
- 当前没有专用 PostgreSQL 测试库，也没有关联的 GitHub Actions 运行，因此真实迁移幂等和数据库集成验收尚未完成。

## 任务状态表

| 任务 ID | 任务名称 | 状态 | 实施记录 | 最小验证 | 阶段回归 |
|---|---|---|---|---|---|
| R0-01 | 工作区状态与基线提交确认 | DONE | [实施记录](./R00-01-工作区状态与基线提交确认.md) | 通过远端基线、分支存在性与分支起点核对；本地工作树检查受环境阻塞并已记录 | 未到阶段出口 |
| R0-02 | 模型保护资产指纹 | DONE | [实施记录](./R00-02-模型保护资产指纹.md) | Node 语法、基准、篡改失败、禁止资产失败和 CRLF 兼容验证通过；完整工作树、Windows 和 CI 未执行项已记录 | 未到阶段出口 |
| R0-03 | 命令契约冻结 | DONE | [实施记录](./R00-03-命令契约冻结.md) | 171 命令三方扫描、Node 语法、完整合成基准及缺失、重复、孤立、动态命令负向验证通过；完整工作树、Windows 和 CI 未执行项已记录 | 未到阶段出口 |
| R0-04 | 数据库基线 | VERIFYING | [实施与验证记录](./R00-04-数据库基线.md) | 46 个迁移连续性、指纹、静态契约、负向用例和安全执行前检通过；专用 PostgreSQL 实跑受阻 | 未到阶段出口 |
| R0-05 | 前端与 Rust 基线 | BLOCKED | 未创建 | 未执行 | 未执行 |
| R0-06 | Windows 基线 | BLOCKED | 未创建 | 未执行 | 未执行 |

## 实际文件变化累计

- 新增 `docs/modular-rewrite/R00-baseline/README.md`。
- 新增 R0-01、R0-02、R0-03 和 R0-04 独立记录。
- 新增模型保护、命令契约和数据库基线三个架构清单。
- 新增对应只读校验器及数据库安全执行器。
- 更新根 `README.md` 的验证命令和模块化重写摘要。
- 未修改生产源码、依赖、配置、公共接口、数据、迁移 SQL、模型实现或用户可观察行为。

## 已确认接口与兼容性变化

无公共命令、DTO、Schema、数据库结构、配置或运行时兼容性变化。R0-04 只冻结既有迁移和测试边界，没有新增或修改迁移。

## 未解决问题

- 当前执行环境没有完整 Git 工作树，新增仓库级校验器尚未在真实工作树执行；已通过 GitHub 逐文件核对、集合扫描和等规模合成夹具替代验证。
- 当前没有 PostgreSQL、Cargo 或专用可清空测试库，R0-04 的迁移幂等、不可变触发器和 18 个数据库集成测试尚未实跑。
- `new-A` 没有 PR；现有 GitHub Actions 只对 `main` push 或 PR 触发，且工作流没有 PostgreSQL service，因此没有可采用的 Actions 数据库结果。
- Windows 原生验证尚未执行。
- 用户设备上的未提交或未跟踪文件仍不可见；远端 `new-A` 操作未覆盖这些本地内容。

## 阶段门禁状态

未通过。R0-01 至 R0-03 已完成；R0-04 保持 VERIFYING；R0-05、R0-06 保持 BLOCKED，未创建 `R00-stage-completion.md`。

## 下一 READY 任务

当前没有 READY 任务。必须先在名称包含 `test` 的专用 PostgreSQL 数据库上通过 R0-04 全部集成测试，才能将 R0-04 标记为 DONE 并开放 R0-05。
