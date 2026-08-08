# R02-08 Domain 根出口收敛实施记录

- 任务状态：`DONE`
- 前置门禁：R2-07 已由用户确认关闭为 `DONE`
- R2-08 开始基线：`4f93297049d773985dbdf0f077a68fc003d6b7d6`
- 最终实现提交：`62b1f622b9c14b33dbaac850812a49c063ccb090`
- 目标平台：Windows
- 主要目标文件：`crates/domain/src/lib.rs`

## 1. 目标

将 Domain 根文件收敛为唯一公共组合出口：只声明业务模块并显式 re-export 已登记公共兼容类型；删除根文件中的默认值实现和 glob 根出口，同时保持 `football_domain::TypeName` 公共路径兼容。

## 2. 实际实施

- `crates/domain/src/lib.rs` 保留 17 个业务模块公开声明。
- 原 17 条 `pub use <module>::*;` 根级 glob re-export 已全部替换为按业务模块分组的显式 `pub use module::{Type...};`。
- 365 个 `publicCompatibilityType` 全部继续保留 `football_domain::TypeName` 根级公共路径。
- 为保持历史根 API，额外登记并显式 re-export 34 个既有公共常量/格式版本符号；公共根兼容不再依赖 glob export。
- `default_true`、`default_team_page_limit`、`default_confidence` 三个私有默认值实现已迁入 `shared/defaults.rs`；根文件只保留 crate 内显式兼容 re-export，不承载实现。
- 新增 `scripts/domain-inventory/root-export-policy.mjs`，从 Domain 类型清单确定唯一模块归属和显式出口集合。
- 新增 `scripts/generate-domain-root-exports.mjs`，可确定性生成根出口。
- 新增 `scripts/verify-domain-root-exports.mjs`，拒绝根级 glob re-export、根文件领域定义/实现、未知模块、重复类型、遗漏类型和与清单不一致的显式出口。
- `npm run verify:architecture` 已接入根出口静态门禁，并新增 `verify:domain-root-exports` 独立命令。
- `inventory-document.mjs` 现在从真实 `lib.rs` 推导当前根出口策略，不再永久把历史 glob 债务写死到清单。

## 3. 生成与静态验证结果

- 临时 Windows 生成 run `31235159425` 成功：显式生成 365 个根级公共兼容类型，rustfmt 后重新生成类型清单，`verify-domain-type-inventory` 与 `verify-domain-root-exports` 均通过，并自动删除临时 workflow。
- 最新 `architecture/domain-type-inventory.json`：365 个类型、365 个公共兼容类型、299 个 PostgreSQL 映射类型、129 个 Domain 来源文件、284 个 Rust 扫描文件。
- 清单中的 `currentPublicExportPolicy` 与 `targetPublicExportPolicy` 现在均为 `explicit re-export only`。
- 临时生成/同步 workflow 已从最终源码树删除，不作为长期项目文件。

## 4. 兼容边界

- 未删除任何已登记公共兼容类型。
- 未改变根级 `football_domain::TypeName` 路径。
- 未改变模块语义路径、Serde、数据库映射、DTO、配置、错误语义或日志。
- 未修改模型实现、模型保护资产、PostgreSQL migration 或生产依赖。

## 5. 当前验证状态

源码切换、确定性静态门禁和 Windows 本机阶段回归均已完成。Windows 专项 run `31236344727` 通过根出口、球队资料包和确定性保护资产门禁，并生成最终实现提交 `62b1f622b9c14b33dbaac850812a49c063ccb090`。用户按阶段回归执行后反馈未见报错；上传 runtime 日志 58 条记录全部为 `info`，`bootstrap` 的 `connection_error` 为 `null`，球队、阵容、分析、Postmatch 与 API 工作区读取均正常完成。R2-08 关闭为 `DONE`。

## 6. 完成标准

- `lib.rs` 不包含领域类型定义、构造器或业务逻辑。
- `lib.rs` 不包含公共 glob re-export。
- 365 个公共兼容类型全部以显式根出口保留。
- 根默认值实现迁离组合根且现有内部调用继续编译。
- Domain 清单与根出口静态门禁通过。
- Windows 最小验证与阶段回归通过。
- 根 README、阶段 README 与本记录同步实际结果。

以上完成标准均已满足；专用可清空 PostgreSQL 测试库基线和私有模型固定回归仍按阶段既有策略保留到最终统一验收，不在本节点伪报为已通过。
