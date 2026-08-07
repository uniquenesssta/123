# R02-06 Review 与 Postmatch 实施记录

- 任务状态：`IMPLEMENTING`
- 前置门禁：R2-05 已在正式 Windows Automated run `31171082098`、job `92842834091` 通过
- R2-06 开始基线：`b0437922f573fe4ee066e4217ac64694da71f34a`
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

计划删除并由新目录唯一替代：

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

新增 R2-06 模块路径身份测试，覆盖 Review 48 + Postmatch 11 共 59 个类型，验证新业务语义路径与既有根级公共路径保持同一 Rust 类型身份。原比赛事件语义、旧事件载荷默认值和资料包工作流状态机单元测试随职责迁移。

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

## 7. 当前结果

当前正在实施。类型清单需要在新路径落地后重新生成；任何专项、全量或保护资产门禁失败都停止，不将 R2-06 标记为 `DONE`。
