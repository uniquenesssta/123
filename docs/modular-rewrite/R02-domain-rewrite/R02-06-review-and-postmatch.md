# R02-06 Review 与 Postmatch 实施记录

- 任务状态：`VERIFYING`
- 前置门禁：R2-05 已在正式 Windows Automated run `31171082098`、job `92842834091` 通过
- R2-06 开始基线：`b0437922f573fe4ee066e4217ac64694da71f34a`
- 当前已验证源码提交：`14ef207c754a73ae53bece3593e241d0d2ea428a`
- 目标平台：Windows
- 类型范围：Review 48 + Postmatch 11，共 59 个公共兼容类型

## 1. 目标

将 Review 与 Postmatch 从根级职责混合文件迁移为唯一业务语义目录，保持根级公共兼容路径、Serde、数据库映射、Application、Tauri DTO、模型保护边界和用户可观察行为不变。

## 2. 实际拆分

### Review

- `review/ability_candidate.rs`：能力候选状态、决策、记录与提案
- `review/result.rs`：赛果草稿与记录
- `review/substitution.rs`：换人草稿与记录
- `review/observation.rs`：球员比赛观测与表现指标
- `review/participant.rs`：球员/球队复盘记录
- `review/aggregate.rs`：复盘草稿、摘要与详情聚合
- `review/preparation.rs`：复盘准备输入与上下文
- `review/calculation.rs`：复盘计算结果
- `review/event/semantics.rs`：比赛事件类型、核验/修订状态和汇总
- `review/event/payload.rs`：复盘事件草稿与持久化记录
- `review/package/`：资料包合同、比较、预览、状态机与提交结果

### Postmatch

- `postmatch/settlement.rs`：正式结算就绪度、草稿与记录
- `postmatch/evidence.rs`：证据裁决与评分记录
- `postmatch/provider_score.rs`：来源提供器评分快照
- `postmatch/monitoring.rs`：赛后漂移与监控请求
- `postmatch/overview.rs`：赛后工作台聚合

`review/mod.rs`、`review/event/mod.rs`、`review/package/mod.rs` 与 `postmatch/mod.rs` 只负责显式模块组合和 re-export。

## 3. 旧实现清理

以下 5 个职责混合源文件已删除，并由新目录唯一替代：

- `crates/domain/src/review.rs`
- `crates/domain/src/match_event.rs`
- `crates/domain/src/match_review_package.rs`
- `crates/domain/src/match_review_workflow.rs`
- `crates/domain/src/postmatch.rs`

## 4. 兼容边界

- 根级 `football_domain::TypeName` 继续保留；R2-08 前不提前清理全局根级兼容 re-export。
- 新增 `football_domain::review::*` 与 `football_domain::postmatch::*` 业务语义路径。
- Serde 字段名、snake_case 枚举值、默认值、optional 语义和历史 JSON 不改。
- 比赛事件解析别名、工作流状态机、阻断原因和默认行为原样迁移。
- 不修改数据库迁移、SQL Row、Application、Tauri DTO、公共命令、生产依赖、P4/P7 模型实现、参数、Profile、Schema、fixture 或 Golden Master。

## 5. 契约测试

新增 `crates/domain/tests/serde_contracts/r2_06_module_paths.rs`，覆盖 Review 48 + Postmatch 11 共 59 个类型，验证新业务语义路径与既有根级公共路径保持同一 Rust 类型身份。原比赛事件语义、旧事件载荷默认值和资料包工作流状态机单元测试随职责迁移。

`architecture/domain-migration-progress.json` 已登记 `R2-06`，`scripts/domain-inventory/target-module-policy.mjs` 已登记 `review/` 与 `postmatch/` 的唯一 R2-06 归属，后续类型清单门禁拒绝回退到目标目录之外。

## 6. 验证计划

- `cargo fmt --all`
- `node scripts/generate-domain-type-inventory.mjs`
- `node scripts/verify-domain-type-inventory.mjs`
- `cargo test --locked -p football-domain --test serde_contracts`
- `npm run verify:architecture`
- `node scripts/verify-protected-assets-deterministic.mjs`
- `npm run verify:frontend`
- `npm run verify:rust`
- 最终源码树执行正式 `Public Platform CI` / Windows Automated
- 用户本机 Windows 验证并行执行，作为额外实机证据；真实 PostgreSQL 与最终 Windows Full 仍按总计划保留到最终统一验收

## 7. 实施与验证结果

- 初始迁移提交 `3e98cd65b7152520df4a379386b6a1864a61c16d` 完成 Review/Postmatch 59 类型职责拆分、旧 5 文件删除、根级兼容 re-export 和模块路径测试接入。
- staged run `31173619041`、job `92850680084` 中，父级依赖准备与 `cargo fmt --all` 通过；随后类型清单生成器硬失败：`R2 目标模块策略缺少来源文件：crates/domain/src/review/ability_candidate.rs`。后续专项和全量门禁按规则停止，未把该运行描述为通过。
- 根因是目标模块策略仍只登记旧根文件，未登记迁移后的 `review/` / `postmatch/` 目录。提交 `e00263d803565b027b32e7023ba881407932e17e` 仅补齐这两个 R2-06 目录策略，没有放宽任何门禁。
- recovery staged run `31173824393`、job `92851309157` 已完整通过：父级依赖准备、Rust 格式化、Domain 类型清单重新生成与验证、R2-06 Serde/模块身份契约、架构门禁、模型保护资产门禁、完整 frontend 和完整 Rust 门禁全部成功。
- 上述成功运行自动生成并提交最终格式化源码树与类型清单，提交为 `14ef207c754a73ae53bece3593e241d0d2ea428a`；临时 staged workflow 已从最终树删除。
- 当前状态为 `VERIFYING`：专项与全量仓库门禁已经通过，但仍需在最终文档同步后的源码树执行正式 `Public Platform CI` / Windows Automated；通过前不开放 R2-07。
