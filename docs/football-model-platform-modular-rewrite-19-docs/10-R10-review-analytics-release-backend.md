# R10：Review、Postmatch、Analytics 与 Release 后端重写——独立执行任务书

> 文档编号：`R10`  
> 前置阶段：`R9`  
> 后续阶段：`R11`  
> 本文档是唯一执行依据之一；必须与 `00-总体架构与前23节.md` 同时适用。

## 当前前置状态（2026-10-13）

R9 全部节点及 Windows Automated 阶段出口 DONE；已验源码 b50d804 / Windows run38221009454 SUCCESS，R9 文档收尾 HEAD48618fed2d7ff7c2a4db831a037f227dc789f278。用户2026-10-12“开始R10-01”授权，唯一新分支 rewrite/r10-review-analytics-release-backend 已建立；R10 IN_PROGRESS，01自身精确838e39e / run38229119264 / job114743919870全SUCCESS并DONE；02精确1ad6e68 / Windows run38245932270/job114799659672全SUCCESS并DONE；用户“收尾02开始03”授权03已实施，精确99b15ad / Windows run38259438025/job114845878326全SUCCESS并DONE，04精确221d211 / Windows run38271387412/job114886215505全SUCCESS并DONE；05精确fe3529f / Windows run38277864089/job114907643831全SUCCESS并DONE；06精确1ed6dbe / Windows run38290328302/job114948593892全SUCCESS并DONE；07精确cdd2797 / Windows run38350692611/job115142166008全SUCCESS并DONE；08精确f834e31 / Windows run38357781835/job115164583225全SUCCESS并DONE；用户“收尾08开始09”授权09 READY，10～13 BLOCKED；[08实际记录](../modular-rewrite/R10-review-analytics-release-backend/R10-08-drift-monitoring.md)；[07实际记录](../modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)；[06实际记录](../modular-rewrite/R10-review-analytics-release-backend/R10-06-evidence-scoring.md)；[05实际记录](../modular-rewrite/R10-review-analytics-release-backend/R10-05-postmatch-settlement.md)；[04实际记录](../modular-rewrite/R10-review-analytics-release-backend/R10-04-ability-candidate-transaction.md)；[03实际记录](../modular-rewrite/R10-review-analytics-release-backend/R10-03-review-engine-adapter.md)。[02实际记录](../modular-rewrite/R10-review-analytics-release-backend/R10-02-structured-match-facts.md)。实际 owner、测试与完整清单见 [01实施记录](../modular-rewrite/R10-review-analytics-release-backend/R10-01-review-package-workflow.md) 及 [本阶段索引](../modular-rewrite/R10-review-analytics-release-backend/README.md)。真实数据库/历史保护/账本并发回滚/XLSX/Full/私有回归及继承删除风险仍最终新库待验，R9成功不替代R10自身验收。

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

状态：`DONE`（838e39e / Windows run38229119264 / job114743919870全SUCCESS；Application146/Persistence179/Gateway112 unit、41 contract及17视口/构建/打包/启动通过）

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

状态：`DONE`（1ad6e68 / Windows run38245932270 / job114799659672全SUCCESS；Application146/Persistence187/Gateway112 unit、41 contract及17视口/构建/打包/启动通过）

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

状态：`DONE`（99b15ad / Windows run38259438025 / job114845878326全SUCCESS；Application151/Persistence187/Gateway112 unit、41 contract及17视口/构建/打包/启动通过，01～03 DONE、04～13 BLOCKED）

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

状态：`DONE`（221d211 / Windows run38271387412 / job114886215505全SUCCESS，App151/PG195/Gateway112unit+41contract、新增8与17视口/打包/启动通过）

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

状态：`DONE`（fe3529f / Windows run38277864089 / job114907643831全SUCCESS，App159/PG199/Gateway112unit+41contract、新增12与17视口/打包/启动通过）

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

状态：`DONE`（1ed6dbe / Windows run38290328302 / job114948593892全SUCCESS，App165/PG207/Gateway112unit+41contract、新增14与17视口/打包/启动通过）

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

状态：`DONE`（cdd2797 / Windows run38350692611/job115142166008完整SUCCESS，App165/PG216/Gateway112unit+41contract及9新unit通过，两轮失败关闭）

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

状态：`DONE`（f834e31 / Windows run38357781835/job115164583225全SUCCESS，App171/PG227/Gateway112unit+41contract及17新增unit通过）

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

状态：`VERIFYING`（用户2026-10-13“收尾08开始09”授权；原计算入口与PG事务职责已重写，84源码/完整架构/等价/负向通过，自身完整Windows待验，10～13BLOCKED）。详见[09实施记录](../modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md)。

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


## R10-03 精确 Windows 验收完成（2026-10-12）

精确提交 `99b15ad261a178322231320ad9dc2f8af00ded8e` / tree `a52fd71ad9551a615720c49d9a69d3380d90361f`，[Public Platform CI run38259438025](https://github.com/uniquenesssta/123/actions/runs/38259438025) / Windows job114845878326 全 SUCCESS，run 于 2026-10-12 22:22:12 北京时间完成。实际 Application151/151、Persistence187/187、Gateway112/112 unit 和41/41 contract；新增5个普通入口测试及迁移的2个资料包生成测试均实际通过。前端契约、TypeScript、Vite、17视口、Rust fmt、locked Clippy all-targets -D warnings、完整workspace tests、Windows release/MSI/NSIS、客户端启动和运行日志扫描全部通过（7条记录、3个完成操作）。

证据 artifact11753201967，14,041,892字节，SHA-256 `984de7735649121de192de496d5d92921feb46583e145ad7b8985d37a483ff04`；报告 `windows-acceptance-20261012-135927.json` 及同名.txt。验收对应上述03自身代码，不借用02结果。本次仅五份既有文档补齐验收状态并提交 [skip ci]，生产源码、测试、门禁、清单及依赖均与精确成功提交一致，不重复运行同源码Windows。

03自身完整Windows通过，当前 DONE；01～03 DONE，04～13 BLOCKED，R10 IN_PROGRESS，继续唯一 `rewrite/r10-review-analytics-release-backend` 分支。用户重复“收尾02开始03”的范围已完成；本次没有新增04授权或进入04实施，没有创建R10阶段完成记录。此前03 VERIFYING、预期151和待Windows描述保留为实施历史，以本节和首部当前状态为准。

真实PG、历史保护四项、不可变账本、并发/回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承的model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub、静态指纹不计这些真实验收PASS。本轮未运行非Windows Cargo/check/Clippy/build/test、loopback或浏览器动态。


## R10-04 实施订正与当前边界（2026-10-12）

R10-04 Ability Candidate Transaction已实施 `VERIFYING`，基线aa24f5a，继续唯一rewrite/r10-review-analytics-release-backend。PG候选列表/详情/行映射/人工决策事务迁入ability_candidate_transaction唯一模块；原状态、处理人归一化、0.01过期判断和四状态解码提取并复用，旧root四职责删除。Application/Ports/公开PostgresStore签名、全部SQL/错误/JSON载荷、候选锁与单事务审计/提交后回读保持；拒绝不写能力观察，不自动接受或晋升参数。新增8原PG unit待Windows，预期Persistence195 / Application151 / Gateway112 unit、41 contract；84/84静态、完整架构、8 Rustfmt、13等价核对与8破坏探针通过。01～03自身Windows DONE，04 VERIFYING、05～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

参考目录仅要求实际候选责任收拢，原Application透传不做机械迁移。唯一实际事务、读/行映射/纯门禁，7A/8M/0D及全部23字段/原并发边界见[04记录](../modular-rewrite/R10-review-analytics-release-backend/R10-04-ability-candidate-transaction.md)；无新runner/target/workflow/数据库专项入口或迁移。


## R10-05 实施订正与当前边界（2026-10-13）

R10-05 Postmatch Settlement已实施 `VERIFYING`，基线96ef21d，继续唯一rewrite/r10-review-analytics-release-backend。Application结算/readiness/列表迁入settlement唯一模块，三个旧用例删除；PG15原生产职责、3上下文类型及1原时效unit迁入具名结算适配器，证据队列/正式样本继续借用同一原事务。全部SQL/错误/JSON、指纹/幂等键、原readiness、既有返回/插入冲突路径及提交后回读保持，原资料包mark_settled仍独立Port；不自动接受候选或晋升参数。新增8 Application/4 PG unit待Windows，预期Application159 / Persistence199 / Gateway112 unit、41 contract；84/84静态、完整架构、18 Rustfmt、12等价与8负向通过。01～04自身Windows DONE，05 VERIFYING、06～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

参考App目录扩展为实际完整结算链：原事务的证据队列/正式样本支持同步迁移，原评分判定/供应商/漂移正文不变。实际15A/14M/3D、23字段/完整兼容/异常/已有与冲突路径差异见[05记录](../modular-rewrite/R10-review-analytics-release-backend/R10-05-postmatch-settlement.md)。无新框架、入口或迁移，06～13未启动。


## R10-06 实施订正与当前边界（2026-10-13）

R10-06 Evidence Scoring已实施 `VERIFYING`，基线18f7264，继续唯一rewrite/r10-review-analytics-release-backend。App两证据用例迁入evidence_scoring并切Service/删旧路径；PG查询/映射/人工决定/评分与原队列收拢唯一owner，结算仍借用原tx调用同一队列方法。原SQL/错误/JSON、说明trim后UTF-8字节门槛、评分权重/别名、不可验证NULL、不可变决定/审计/提交后回读保持；不自动判定/晋升参数。新增6App/8PG unit待Windows，预期Application165 / Persistence207 / Gateway112 unit、41 contract；84/84静态、完整架构、16 Rustfmt、16等价及8负向通过。01～05自身Windows DONE，06 VERIFYING、07～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

参考evidence-scoring采用Rust语义evidence_scoring；实际PG整评分链与05原队列同责任迁入、原事务调用不变。12A/13M/3D、23字段/字节语义/全部兼容/风险/回退见[06记录](../modular-rewrite/R10-review-analytics-release-backend/R10-06-evidence-scoring.md)。未启动07～13。


## R10-07 实施订正与当前边界（2026-10-13）

R10-07 Provider Score已实施 `VERIFYING`，基线85ecf15，继续唯一rewrite/r10-review-analytics-release-backend。实际PG供应商聚合、原评分/指纹与快照写入、最新读取和字段映射归provider_score唯一owner；App综合监控/两原Ports不变，不为参考App目录虚构供应商入口。原SQL五分区过滤/BTreeMap与决定顺序、60/15/25原公式、空准确率零分、未先舍入的均值、原指纹/版本/快照键及逐组pool幂等写入保持；监控仍供应商→漂移→总览，没有新事务/审计或自动参数晋升。新增9原PG unit待自身Windows，预期Persistence216 / Application165 / Gateway112 unit、41 contract；84/84静态、完整架构、8 Rustfmt、14等价及10负向通过。01～06自身Windows DONE，07 VERIFYING、08～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

参考App provider-score目录与真实来源不同：仅有综合Monitoring Port，实际PG评分职责收拢provider_score，App保持原综合入口/契约。7A/9M/0D、23字段/全部兼容/共享分区内部可见性/风险/回退见[07记录](../modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)。未启动08～13。


## R10-07 Windows 修复记录（2026-10-13）

首轮688892d/run38296053769在Clippy uninlined_format_args失败，前端/17视口通过，Rust单测/打包/启动未完成。原快照键改捕获四具名值，内容与原接口/SQL/公式保持；旧格式回归守卫、使用摘要和全部记录同步，84静态/完整architecture/1Rustfmt/等价及13探针通过。9unit仍待本修复Windows，07VERIFYING、08～13BLOCKED；完整修复9M与当前累计7A/10M/0D见[07记录](../modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)。


## R10-07 第二轮夹具修复记录（2026-10-13）

8c9ab5d/run38346437982实际frontend/17视口/fmt/Clippy/App165通过，PG215通过1精度测试失败：旧夹具算式为.2641972900而预期误写.264198，且无法区分提前舍入。现原unit改两个经Decimal/整数独立核对的区分夹具，补静态数据守卫，生产公式/链/其他8unit/容差保持；修复8M，累计7A/10M/0D，84源码/完整architecture/1Rustfmt与3夹具负向通过。07VERIFYING、08～13BLOCKED，完整[07记录](../modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)。


## R10-08 实施订正与当前边界（2026-10-13）

R10-08 Drift Monitoring已实施 `VERIFYING`，基线33f7188，继续唯一rewrite/r10-review-analytics-release-backend。App原综合监控两入口归monitoring；PG监控前检/活动分区/总览与漂移样本/纯准备/幂等事务/读取映射各有唯一owner。原Monitoring Port与公开签名保持；供应商→漂移→总览顺序、全部SQL/原纯引擎、窗口/状态/指纹、单漂移事务、新run才写finding/审计与冲突不重写保持。新增6App/11PG unit待自身Windows，预期Application171 / Persistence227 / Gateway112unit+41contract；84/84静态、完整架构、22Rustfmt、19等价及10负向通过。01～07自身Windows DONE、08VERIFYING、09～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

参考drift-monitoring采用PG drift_monitoring实际职责，App/PG综合监控采用monitoring唯一编排与共享partition；不创建新Port或改变供应商/漂移/总览顺序。18A/13M/2D及全部契约/异常/回退见[08记录](../modular-rewrite/R10-review-analytics-release-backend/R10-08-drift-monitoring.md)。没有进入09。
