# R00 基线冻结与可重复验收：执行记录索引

## 阶段范围

冻结模块化重写开始前的远端提交、分支、保护资产、公共命令、数据库、前后端与 Windows 验收基线。R00 不实施业务重写，不移动源码，不修改模型保护区。

## 当前基线

- 基线分支：`main`
- 起始基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- R0-02 起始提交：`beb00b6f723d75703e9235967c4f49f985d41c4e`
- R0-03 起始提交：`83a7d0bddd83894b4f551725c40b50705470e731`
- R0-04 起始提交：`8cddf452e01770a042b671ed8c11bc29afd2f1b1`
- R0-05 起始提交：`92976da2372287a66f91d31da6d4f090734dee6c`
- R0-05 CI 测试提交：`94f86842db227c5153d4334ad5176159a840e429`
- 实施分支：`new-A`
- 已完成节点：`R0-01`、`R0-02`、`R0-03`、`R0-04（数据库实跑延期）`、`R0-05`

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
- 数据库公共接口、迁移集合和不可变约束继续以 `main` 基线提交 `db79995873460688c15abb3497bf1c61b73ffb18` 保持不变。
- `scripts/verify_database_baseline.mjs` 与 `scripts/run_database_baseline.mjs` 已建立静态门禁和安全执行入口。
- 用户于 2026-08-04 明确要求将真实 PostgreSQL 迁移幂等和集成测试推迟到最终统一验证；未执行项不得描述为通过。

## 前端与 Rust 基线

- 通过关闭且未合并的 Draft PR #1 触发 `Public Platform CI`，workflow run 为 `30910130867`。
- `npm ci` 通过；存在 1 个 moderate npm audit 警告。
- `npm run verify:frontend` 失败于 GitHub Ubuntu Chromium 未开放调试端口，未进入截图像素或业务断言比较。
- Rust 工具链、Tauri Linux 依赖、公开模型边界、Cargo.lock 与 locked metadata 通过。
- `cargo fmt --all -- --check` 失败；Clippy 和 workspace tests 因 fail-fast 未执行。
- Actions 没有按 package script 单入口直接执行完整 `npm run verify:rust`，差异已在节点记录中列明。
- R0-05 没有修改业务源码、依赖、配置或工作流来掩盖现有失败。

## 任务状态表

| 任务 ID | 任务名称 | 状态 | 实施记录 | 最小验证 | 阶段回归 |
|---|---|---|---|---|---|
| R0-01 | 工作区状态与基线提交确认 | DONE | [实施记录](./R00-01-工作区状态与基线提交确认.md) | 通过远端基线、分支存在性与分支起点核对；本地工作树检查受环境阻塞并已记录 | 未到阶段出口 |
| R0-02 | 模型保护资产指纹 | DONE | [实施记录](./R00-02-模型保护资产指纹.md) | Node 语法、基准、篡改失败、禁止资产失败和 CRLF 兼容验证通过；完整工作树、Windows 和 CI 未执行项已记录 | 未到阶段出口 |
| R0-03 | 命令契约冻结 | DONE | [实施记录](./R00-03-命令契约冻结.md) | 171 命令三方扫描、Node 语法、完整合成基准及缺失、重复、孤立、动态命令负向验证通过；完整工作树、Windows 和 CI 未执行项已记录 | 未到阶段出口 |
| R0-04 | 数据库基线 | DONE | [实施与验证记录](./R00-04-数据库基线.md) | 46 个迁移连续性、指纹、静态契约、负向用例和安全执行前检通过；真实 PostgreSQL 按用户要求延期到最终验证 | 未到阶段出口；数据库实跑缺口保留 |
| R0-05 | 前端与 Rust 基线 | DONE | [实施与验证记录](./R00-05-前端与Rust基线.md) | `npm ci` 通过；前端 Chromium 启动失败；Rust fmt 失败；Clippy/tests 未执行；全部事实已记录 | 未到阶段出口；现有失败已冻结 |
| R0-06 | Windows 基线 | READY | 未创建 | 未执行 | 未执行 |

## 实际文件变化累计

- 新增 `docs/modular-rewrite/R00-baseline/README.md`。
- 新增 R0-01 至 R0-05 独立记录。
- 新增模型保护、命令契约和数据库基线三个架构清单。
- 新增对应只读校验器及数据库安全执行器。
- 更新根 `README.md` 的验证命令和模块化重写摘要。
- 为 R0-05 创建并关闭 Draft PR #1；未合并到 `main`。
- 未修改生产源码、依赖、配置、公共接口、数据、迁移 SQL、模型实现或用户可观察行为。

## 已确认接口与兼容性变化

无公共命令、DTO、Schema、数据库结构、配置或运行时兼容性变化。R0-04 只冻结既有数据库边界；R0-05 只记录现有前端和 Rust 验证基线。

## 未解决问题

- GitHub Ubuntu Chromium 截图进程未开放调试端口，Linux UI 截图基线未完成。
- Rust 工作区 `cargo fmt --check` 失败；可用连接器没有返回完整格式 diff。
- Clippy、workspace tests 和精确 `npm run verify:rust` 尚未形成完整结果。
- 数据库真实迁移幂等、不可变触发器和 18 个 PostgreSQL 集成测试按用户要求延期到最终统一验证。
- Windows 原生验证尚未执行。
- 用户设备上的未提交或未跟踪文件仍不可见；远端 `new-A` 操作未覆盖这些本地内容。

## 阶段门禁状态

未通过。R0-01 至 R0-05 已完成基线记录，其中数据库实跑缺口和 R0-05 现有失败均已明确保留；R0-06 尚未完成，未创建 `R00-stage-completion.md`。

## 下一 READY 任务

唯一允许进入实施的下一节点为 `R0-06 Windows 基线`。
