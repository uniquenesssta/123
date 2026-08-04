# R00 基线冻结与可重复验收：执行记录索引

## 阶段范围

冻结模块化重写开始前的远端提交、分支、保护资产、公共命令、数据库、前后端与 Windows 验收基线。R00 不实施业务重写，不移动源码，不修改模型保护区。

## 当前基线

- 基线分支：`main`
- 起始基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- R0-02 起始提交：`beb00b6f723d75703e9235967c4f49f985d41c4e`
- R0-03 起始提交：`83a7d0bddd83894b4f551725c40b50705470e731`
- 实施分支：`new-A`
- 已完成节点：`R0-01`、`R0-02`、`R0-03`

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

## 任务状态表

| 任务 ID | 任务名称 | 状态 | 实施记录 | 最小验证 | 阶段回归 |
|---|---|---|---|---|---|
| R0-01 | 工作区状态与基线提交确认 | DONE | [实施记录](./R00-01-工作区状态与基线提交确认.md) | 通过远端基线、分支存在性与分支起点核对；本地工作树检查受环境阻塞并已记录 | 未到阶段出口 |
| R0-02 | 模型保护资产指纹 | DONE | [实施记录](./R00-02-模型保护资产指纹.md) | Node 语法、基准、篡改失败、禁止资产失败和 CRLF 兼容验证通过；完整工作树、Windows 和 CI 未执行项已记录 | 未到阶段出口 |
| R0-03 | 命令契约冻结 | DONE | [实施记录](./R00-03-命令契约冻结.md) | 171 命令三方扫描、Node 语法、完整合成基准及缺失、重复、孤立、动态命令负向验证通过；完整工作树、Windows 和 CI 未执行项已记录 | 未到阶段出口 |
| R0-04 | 数据库基线 | READY | 未创建 | 未执行 | 未执行 |
| R0-05 | 前端与 Rust 基线 | BLOCKED | 未创建 | 未执行 | 未执行 |
| R0-06 | Windows 基线 | BLOCKED | 未创建 | 未执行 | 未执行 |

## 实际文件变化累计

- 新增 `docs/modular-rewrite/R00-baseline/README.md`。
- 新增 R0-01、R0-02 和 R0-03 独立实施记录。
- 新增 `architecture/protected-assets.json` 与 `scripts/verify_protected_assets.mjs`。
- 新增 `architecture/command-contract.json` 与 `scripts/verify_command_contract.mjs`。
- 更新根 `README.md` 的验证命令和模块化重写摘要。
- 未修改生产源码、依赖、配置、公共接口、数据、迁移、模型实现或用户可观察行为。

## 已确认接口与兼容性变化

无公共命令、DTO、Schema、数据库、配置或运行时兼容性变化。R0-03 只冻结当前 171 个命令及其签名定位和模块归属，没有修改现有实现。

## 未解决问题

- 当前执行环境无法建立完整 Git 工作树，不能直接运行仓库级 `node scripts/verify_protected_assets.mjs` 与 `node scripts/verify_command_contract.mjs`；已通过 GitHub 逐文件核对、集合扫描和等规模合成夹具替代验证。
- Windows 原生和 GitHub Actions 尚未实际执行新增校验器，将在用户环境、后续 PR 或 R17 统一验证阶段复核。
- 用户设备上的未提交或未跟踪文件仍不可见；远端 `new-A` 操作未覆盖这些本地内容。

## 阶段门禁状态

未通过。R0-01、R0-02、R0-03 已完成；R0-04 至 R0-06 尚未完成，未创建 `R00-stage-completion.md`。

## 下一 READY 任务

唯一允许进入实施的下一节点为 `R0-04 数据库基线`。
