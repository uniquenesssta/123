# R02-07 Analytics、Exchange、AI 与 Release 实施记录

- 任务状态：`VERIFYING`
- 前置门禁：R2-06 已关闭为 `DONE`
- R2-07 开始基线：`582bc078b7e12d8eda38820f44696953d7c11d29`
- 首次迁移提交：`90867616972275849b705e620b830619378df0b8`
- 目标平台：Windows
- 类型范围：Analytics 39 + Exchange 54 + AI Workspace 16 + Release 9，共 118 个公共兼容类型

## 1. 目标

将剩余 Analytics、Exchange、AI Workspace 与 Release Domain 对象从职责混合根文件迁移到唯一业务语义目录，保持根级公共兼容路径、Serde、数据库映射、Application、Tauri DTO、模型保护边界和用户可观察行为不变。

## 2. 实际拆分

### Analytics

- `analytics/calculation.rs`：分析刷新请求、评估样本、校准、模型比较、漂移与聚合结果
- `analytics/quality.rs`：数据质量发现、汇总与处置决策
- `analytics/query_performance.rs`：查询性能发现与汇总
- `analytics/job.rs`：后台任务状态、记录与入队草稿
- `analytics/ai_analysis/`：AI 分析包与响应/建议契约
- `analytics/parameter/`：参数调优、就绪度、影子验证、晋升与回滚契约

### Exchange

- `exchange/dynamic_tag.rs`：球员动态标签交换对象
- `exchange/contribution.rs`：比赛贡献输入、分量与结果
- `exchange/lineup.rs`：比赛阵容导入导出契约
- `exchange/ai_match.rs`：AI 比赛资料包契约
- `exchange/prediction_input.rs`：准备后的比赛预测输入
- `exchange/spreadsheet/`：Excel 导入契约、预览/提交结果与导出数据行
- `exchange/monthly/`：月度工作簿合同、数据缺口与球队月度数据
- `exchange/team_package/`：球队完整资料包导出、覆盖率、预览与提交结果

### AI Workspace

- `ai_workspace/preset.rs`：工作台预设
- `ai_workspace/attachment.rs`：附件值对象
- `ai_workspace/session.rs`：会话草稿、记录与详情
- `ai_workspace/message.rs`：消息草稿与记录
- `ai_workspace/operation.rs`：提议操作、持久化记录与应用结果
- `ai_workspace/file.rs`：生成文件草稿、记录与内容
- `ai_workspace/assistant.rs`：助手输出 Schema、操作和文件契约

### Release

- `release/contract.rs`：发布验收常量与状态
- `release/request.rs`：验收请求与默认窗口
- `release/check.rs`：验收检查项和分类汇总
- `release/metrics.rs`：性能与成本汇总
- `release/run.rs`：验收运行与摘要
- `release/runtime.rs`：运行时事实快照

各目录 `mod.rs` 只负责模块组合与 re-export，不承载业务逻辑。

## 3. 旧实现清理

首次迁移提交删除以下 7 个旧职责混合源文件，由新目录唯一替代：

- `crates/domain/src/analytics.rs`
- `crates/domain/src/api_workspace.rs`
- `crates/domain/src/exchange.rs`
- `crates/domain/src/spreadsheet.rs`
- `crates/domain/src/monthly_workbook.rs`
- `crates/domain/src/team_package.rs`
- `crates/domain/src/release_acceptance.rs`

## 4. 兼容边界

- 根级 `football_domain::TypeName` 继续保留；R2-08 前不提前收敛根级兼容 re-export。
- 新增 `football_domain::analytics::*`、`exchange::*`、`ai_workspace::*`、`release::*` 业务语义路径。
- Serde 字段名、snake_case 枚举值、默认值、optional 语义和历史 JSON 不改。
- 常量字符串、格式版本、默认数值和状态映射原样迁移。
- 不修改 PostgreSQL migration、Application/Tauri 公共 DTO、公共命令、生产依赖或 P4/P7 模型实现与保护资产。

## 5. 契约测试

新增 `crates/domain/tests/serde_contracts/r2_07_module_paths.rs`，覆盖 118 个 R2-07 公共兼容类型的新业务模块路径与既有根级公共路径类型身份一致性。

`architecture/domain-migration-progress.json` 已登记 `R2-07`；目标模块策略新增 `analytics/`、`exchange/`、`ai_workspace/`、`release/` 的 R2-07 唯一归属。

## 6. 当前验证状态

- Windows 本机首次最小验证已执行。
- `cargo test --locked -p football-domain --test serde_contracts`：17/17 通过，R2-07 四组模块身份契约均通过。
- `cargo fmt --all -- --check`：首次失败，仅发现 `exchange/spreadsheet/import.rs` 与 `r2_07_module_paths.rs` 的 rustfmt 排版差异，无编译或契约失败。
- 已按 rustfmt 输出修正上述两处排版，提交为 `982e0e87c0186c1347488dc73a2e6f83e535584e` 与 `06a75da6e8bc6c93a080d4557307d8856a8e29f7`；未修改业务逻辑、Serde、公共接口或依赖。
- 当前等待 Windows 本机重新执行 `cargo fmt --all -- --check`；通过前保持 `VERIFYING`，不进入下一门禁。

## 7. 待完成门禁

- `cargo fmt --all -- --check`（修复后重跑）
- `node scripts/generate-domain-type-inventory.mjs`
- `node scripts/verify-domain-type-inventory.mjs`
- `npm run verify:architecture`
- `node scripts/verify-protected-assets-deterministic.mjs`
- `npm run verify:frontend`
- `npm run verify:rust`
- Windows `npm run tauri:dev` 启动与关键页面 smoke test

当前不得标记为 `DONE`。
