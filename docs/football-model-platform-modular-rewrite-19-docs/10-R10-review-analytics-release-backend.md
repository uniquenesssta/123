# R10：Review、Postmatch、Analytics 与 Release 后端重写——独立执行任务书

> 文档编号：`R10`  
> 前置阶段：`R9`  
> 后续阶段：`R11`  
> 本文档是唯一执行依据之一；必须与 `00-总体架构与前23节.md` 同时适用。

## 1. 阶段目标

- 完成从复盘事实到赛后结算、分析、候选晋升和发布验收的后端闭环。

## 2. 本阶段解决的实际问题

- 该闭环包含状态机、不可变事实、幂等结算、后台任务和人工门禁，不能压缩为大 Service。

## 3. 前置输入与进入条件

- R9 完成。
- Review/Analytics engine 纯计算契约稳定。

## 4. 明确范围

### 4.1 纳入范围

- Review package/facts/engine adapter。
- Ability candidate transaction。
- Settlement/evidence/provider/drift。
- Analytics calculations/jobs/package/parameter lifecycle。
- Release acceptance。

### 4.2 排除范围

- 对应 UI。
- 修改纯计算公式。

## 5. 当前实现来源与扫描重点

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

## 6. 目标目录总览

```text
crates/application/src/use_cases/review/
crates/application/src/use_cases/postmatch/
crates/application/src/use_cases/analytics/
crates/application/src/use_cases/release/
crates/persistence-postgres/src/adapters/review/
crates/persistence-postgres/src/adapters/postmatch/
crates/persistence-postgres/src/adapters/analytics/
crates/persistence-postgres/src/adapters/release/
```

## 7. 目录与文件边界规则

- 状态机定义单一来源。
- 事实提交、包提交、候选接受、参数晋升、发布报告分别使用具名事务。
- 后台 Job 只编排，不嵌入计算公式。

## 8. 数据流、调用流与依赖方向

```text
facts -> review workflow -> engine adapter -> results -> candidate transaction
result + evidence -> idempotent settlement -> provider/drift updates
analytics job -> pure engine -> immutable analysis package -> human promotion gate
acceptance inputs -> gates -> immutable release report
```

## 9. 状态所有权与事务边界

- Review workflow 状态由后端 capability/state machine 唯一持有。
- 后台 Job 状态由 Job Repository 唯一持有。
- 参数当前版本由参数生命周期账本持有。

## 10. 公共契约与兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

## 11. 风险与禁止事项

- 不得自动接受能力候选或参数候选。
- 合成 fixture 不得混入真实统计。

## 本阶段 docs 实施记录目录

本阶段使用固定目录：

```text
docs/modular-rewrite/R10-review-analytics-release-backend/
├─ README.md
├─ R10-01-review-package-workflow.md
├─ R10-02-structured-match-facts.md
├─ R10-03-review-engine-adapter.md
├─ R10-04-ability-candidate-transaction.md
├─ R10-05-postmatch-settlement.md
├─ R10-06-evidence-scoring.md
├─ R10-07-provider-score.md
├─ R10-08-drift-monitoring.md
├─ R10-09-analytics-calculation.md
├─ R10-10-background-jobs.md
├─ R10-11-analysis-package.md
├─ R10-12-parameter-lifecycle.md
├─ R10-13-release-acceptance.md
└─ R10-stage-completion.md
```

执行要求：

- 第一个节点进入 `READY` 前创建本目录和 `README.md`。
- 每完成一个节点，立即创建对应记录文件，不得等到阶段结束后集中补写。
- 每个记录必须基于真实 `git diff --name-status`、实际目录树和真实验证结果填写。
- 每次节点状态变化都同步更新本阶段 `README.md`。
- 所有节点完成后创建阶段完成记录；该文件缺失时，本阶段不得通过出口门禁。
- 根 `README.md` 只保存摘要和记录链接，详细变更以本目录为准。


## 全阶段强制执行规则

1. 本阶段只允许修改本阶段明确列出的目标模块，不得夹带无关重命名、格式化、依赖升级或功能变化。
2. 模型保护区 `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 以及关联参数、Profile、Schema、fixture、Golden Master 不得修改。
3. 任何原文件出现第二个独立职责时，必须把该职责模块升级为目录：原职责迁入具名文件，新职责进入新的具名文件；不得继续向原文件追加。
4. 不得使用 `old`、`new`、`legacy`、`copy`、`final`、`v2` 作为长期文件名或目录名。所谓旧职责和新职责必须使用真实业务语义命名。
5. 每条公共 Tauri 命令、DTO 字段、数据库格式、配置键、错误语义、日志等级和用户可观察行为默认保持兼容。
6. 每个业务状态只能有一个所有者；View 不拥有业务状态，API 不拥有页面状态，Repository 不拥有工作流状态。
7. 新旧实现只能在单个 Atomic Task 的受控切换窗口内短暂共存；任务结束前必须切换唯一入口并删除旧实现。
8. 不得新增生产依赖。确有必要时，必须单独提交依赖评估，不得混入业务任务。
9. 每个 Atomic Task 必须先通过最小验证，再运行阶段回归；硬性验证失败立即停止，不得进入下一任务。
10. 实际源码、配置、接口或行为发生变化时，同步更新根目录 `README.md`，只记录实际完成和实际验证结果。
11. 每个节点完成时必须创建 `docs/modular-rewrite/R10-review-analytics-release-backend/<task-record>.md` 并更新阶段 `README.md`；阶段完成时必须创建 `R10-stage-completion.md`。缺少记录不得标记为 `DONE`。

## 原计划阶段摘要（保留用于追溯）

## R10：Review / Postmatch / Analytics / Release 后端重写

Atomic Tasks：

- R10-01 review package workflow
- R10-02 structured match facts
- R10-03 review engine adapter
- R10-04 ability candidate transaction
- R10-05 postmatch settlement
- R10-06 evidence scoring
- R10-07 provider score
- R10-08 drift monitoring
- R10-09 analytics calculation
- R10-10 background jobs
- R10-11 analysis package
- R10-12 parameter lifecycle
- R10-13 release acceptance

关键验收：

- 九步状态机。
- 幂等结算。
- 人工接受能力候选。
- 人工参数晋升。
- 时间切分。
- 合成 fixture 隔离。
- 不可变发布报告。

---

# Atomic Tasks

## R10-01 Review Package Workflow

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Review Package Workflow 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/review/package-workflow/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-01 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-01 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-01-review-package-workflow.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-01-review-package-workflow.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-02 Structured Match Facts

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Structured Match Facts 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/review/facts/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-02 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-02 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-02-structured-match-facts.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-02-structured-match-facts.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-03 Review Engine Adapter

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Review Engine Adapter 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/review/engine-adapter/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-03 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-03 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-03-review-engine-adapter.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-03-review-engine-adapter.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-04 Ability Candidate Transaction

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Ability Candidate Transaction 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/persistence-postgres/src/adapters/review/ability_candidate_transaction/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-04 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-04 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-04-ability-candidate-transaction.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-04-ability-candidate-transaction.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-05 Postmatch Settlement

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Postmatch Settlement 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/postmatch/settlement/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-05 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-05 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-05-postmatch-settlement.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-05-postmatch-settlement.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-06 Evidence Scoring

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Evidence Scoring 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/postmatch/evidence-scoring/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-06 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-06 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-06-evidence-scoring.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-06-evidence-scoring.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-07 Provider Score

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Provider Score 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/postmatch/provider-score/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-07 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-07 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-07-provider-score.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-08 Drift Monitoring

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Drift Monitoring 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/postmatch/drift-monitoring/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-08 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-08 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-08-drift-monitoring.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-08-drift-monitoring.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-09 Analytics Calculation

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Analytics Calculation 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/analytics/calculation/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-09 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-09 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-09-analytics-calculation.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-10 Background Jobs

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Background Jobs 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/analytics/jobs/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-10 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-10 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-10-background-jobs.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-10-background-jobs.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-11 Analysis Package

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Analysis Package 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/analytics/analysis-package/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-11 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-11 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-11-analysis-package.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-11-analysis-package.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-12 Parameter Lifecycle

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Parameter Lifecycle 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/analytics/parameter-lifecycle/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-12 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-12 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-12-parameter-lifecycle.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-12-parameter-lifecycle.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R10-13 Release Acceptance

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Release Acceptance 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/review-engine/`、`analytics-engine/`、`analysis-package/` 的适配调用。
- application review/postmatch/analytics/release 模块。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/release/acceptance/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 九步状态机、幂等键、候选人工接受、参数人工晋升和报告不可变语义不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R10-13 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R10-13 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R10-review-analytics-release-backend/R10-13-release-acceptance.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R10-13-release-acceptance.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

# 阶段级验证矩阵

| 验证层级 | 必须执行 | 通过条件 |
|---|---|---|
| 状态机 | review workflow tests | 九步转换合法 |
| 幂等 | settlement integration | 重复输入无重复副作用 |
| 候选 | manual gate tests | 未确认不生效 |
| 分析 | time split + fixture isolation | 无数据泄漏 |
| 发布 | immutable report tests | 报告不可变 |

# 阶段出口门禁

- `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` 已完整索引全部节点记录。
- `R10-stage-completion.md` 已创建并确认本阶段真实变更、验证、限制和回退点。
- 后端核心闭环全部通过。
- R11 可进入 READY。

# 阶段提交与回退

- 阶段内每个可独立验收的任务保留原子提交；阶段完成提交建议为 `rewrite: complete R10 review-analytics-release-backend`。
- 只允许回退到最近一个通过全部门禁的提交。
- 不得通过保留双实现代替可回退提交。

# 阶段完成记录要求

本阶段所有 Atomic Task 完成并通过阶段回归后，创建：

```text
docs/modular-rewrite/R10-review-analytics-release-backend/R10-stage-completion.md
```

必须使用以下结构：

```text
# R10 阶段完成记录

## 1. 阶段目标与完成结论
## 2. 已完成节点索引
| 任务 ID | 实施记录 | 完成状态 | 最小验证 |

## 3. 实际新增文件总表
## 4. 实际修改文件总表
## 5. 实际移动或重命名文件总表
## 6. 实际删除文件总表
## 7. 最终目录与职责边界
## 8. 最终调用流、数据流和状态所有权
## 9. 公共接口、DTO、Schema、数据与配置变化
## 10. 保持不变的兼容行为
## 11. 旧实现、重复实现和临时路径清理结果
## 12. 阶段级验证与真实结果
## 13. 未执行验证、环境阻塞和剩余风险
## 14. 根 README、阶段 README 与架构文档同步
## 15. 阶段回退点与回退步骤
## 16. 出口门禁逐项结论
## 17. 下一阶段唯一 READY 任务
## 18. 订正记录
```

阶段完成记录必须引用本阶段每个节点记录，不得只重复任务书中的计划。缺少任何节点记录、真实文件总表、验证结果或回退信息时，本阶段不得标记为 `DONE`。
