# R02-08 Domain 根出口收敛实施记录

- 任务状态：`IN_PROGRESS`
- 前置门禁：R2-07 已由用户确认关闭为 `DONE`
- R2-08 开始基线：`4f93297049d773985dbdf0f077a68fc003d6b7d6`
- 目标平台：Windows
- 目标文件：`crates/domain/src/lib.rs`

## 1. 目标

将 Domain 根文件收敛为唯一公共组合出口：只声明业务模块并显式 re-export 已登记公共兼容类型；删除根文件中的默认值实现和 glob 根出口，同时保持 `football_domain::TypeName` 公共路径兼容。

## 2. 当前基线

- `crates/domain/src/lib.rs` 当前声明 17 个业务模块。
- 根文件仍使用 17 条 `pub use <module>::*;` glob re-export。
- 根文件仍直接定义 `default_true`、`default_team_page_limit`、`default_confidence` 三个私有默认值函数。
- `architecture/domain-type-inventory.json` 登记 365 个公共兼容类型，目标公共出口策略为 `explicit re-export only`。

## 3. 实施范围

- 将 365 个公共兼容类型按唯一 `targetModule` 生成显式根级 `pub use module::{...};` 列表。
- 保持 17 个业务模块公开声明不变。
- 将三个根级默认值函数迁入 `shared/defaults.rs`；根文件只保留 crate 内显式兼容 re-export，不继续承载实现。
- 增加根出口静态门禁：拒绝 `pub use ...::*` 回归，并验证根级显式公共类型集合与 Domain 清单一致。
- 重新生成 Domain 类型清单，确保 R2-08 后来源文件、调用方和摘要与真实源码树一致。

## 4. 兼容边界

- 不删除任何已登记公共兼容类型。
- 不改变根级 `football_domain::TypeName` 路径。
- 不改变模块语义路径、Serde、数据库映射、DTO、配置、错误语义或日志。
- 不修改模型实现、模型保护资产、PostgreSQL migration 或生产依赖。

## 5. 当前验证状态

尚未进入最小验证。完成源码切换后由 Windows 本机依次执行格式、Serde、类型清单、架构、protected assets、frontend、Rust 与 Tauri smoke；任一硬门禁失败即保持本任务未完成。

## 6. 完成标准

- `lib.rs` 不包含领域类型定义、构造器或业务逻辑。
- `lib.rs` 不包含公共 glob re-export。
- 365 个公共兼容类型全部以显式根出口保留。
- 根默认值实现迁离组合根且现有内部调用继续编译。
- Domain 清单与根出口静态门禁通过。
- Windows 最小验证与阶段回归通过。
- 根 README、阶段 README 与本记录同步实际结果。
