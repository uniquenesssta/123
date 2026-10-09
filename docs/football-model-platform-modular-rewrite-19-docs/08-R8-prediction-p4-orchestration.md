# R8：Prediction 与 P4 外围链路重写——独立执行任务书

> 文档编号：`R08`  
> 前置阶段：`R7`  
> 后续阶段：`R9`  
> 本文档是唯一执行依据之一；必须与 `00-总体架构与前23节.md` 同时适用。

## 当前执行订正（2026-10-09）

用户已启动唯一 R8 分支 `rewrite/r8-prediction-p4-orchestration`，起点 `90680bf945fbb0d1c191937c2d9e90c2c916fb00`。R7-01～15 及 Windows Automated 已完成；最终代码 `a928c8b` / run `36871154039` 通过。动态 PG、历史四项/账本、有效 XLSX、Windows Full 及模型历史删除风险仍最终封包新库待验，不继承为 PASS。

适用总纲顶部的执行订正：仅 Windows 动态验证，沿用原单测/contract、既有 Windows runner/workflow/数据库入口；不新增专项或持续回归体系。公开仓库没有私有 P4/P7 引擎、参数和 Golden Master；模型保护资产可验证，公共 unavailable stub 的通过不能冒充私有固定概率实跑。目录模板按实际职责及 Rust 模块命名调整；顺序执行的后端 use case 无新增 UI 生命周期，不强制新增 State、请求 ID 或空出口。

R8-01～06 已取得各自精确 Windows CI（`cba72fd` / `36881256338`、`cc2b0fe` / `36891488571`、`47ba3de` / `36902069546`、`59c5679` / `36966815323`、修复 `2aa99a7` / `36971419476`、`0969331` / `37752995641`）及正式完成记录。07 精确 `58b390a` / `37796909083` 已通过并DONE，08精确修复 `52f23ab` / `37817443918` 全SUCCESS并DONE，09精确 `4025781` / `37825126808` 全SUCCESS并DONE，用户已授权“收尾09 开始10”；各项精确当前状态只由 [阶段索引](../modular-rewrite/R08-prediction-p4-orchestration/README.md) 维护。07 及后续必须取得自身 Windows CI，不继承前项 PASS。

## 1. 阶段目标

- 重写模型输入构建、历史特征读取、readiness、manifest、模型执行适配、运行持久化和 P4 外围编排。
- 在不修改模型的前提下保持所有固定回归和时间窗口。

## 2. 本阶段解决的实际问题

- 预测外围链路跨 Domain、Application、Persistence 与模型接口，容易在重写时误改 cutoff、路由或固定概率。

## 3. 前置输入与进入条件

- R7 完成。
- 模型指纹、P4/P7 contract 和 engine regression 基线可重复。

## 4. 明确范围

### 4.1 纳入范围

- Input builder。
- Historical feature reader。
- Readiness audit。
- Deterministic manifest。
- Route/model request。
- Model adapter。
- Run persistence。
- P4 evidence/fact/horizon/workbench/freeze。

### 4.2 排除范围

- model-api/model-p4/model-p7 内部代码、参数、公式和 schema。
- 前端 Prediction UI。

## 5. 当前实现来源与扫描重点

- `crates/application/src/prediction.rs`。
- `p4_orchestration.rs`、`fact_pipeline.rs`、`p4_persistence.rs`、`p4_workbench.rs`。
- 相关 persistence modules。

## 6. 目标目录总览

```text
crates/application/src/use_cases/prediction/
crates/application/src/services/prediction/
crates/application/src/use_cases/research/p4/
crates/persistence-postgres/src/adapters/prediction/
crates/persistence-postgres/src/adapters/p4/
```

## 7. 目录与文件边界规则

- 模型调用只通过 `model-api::PredictionModel`。
- 输入 manifest 必须可重现。
- readiness 只审计，不静默修复缺失数据。
- 正式和影子路径分离。

## 8. 数据流、调用流与依赖方向

```text
match context -> historical features -> readiness -> deterministic manifest -> route -> model-api -> output -> run ledger
P4 evidence -> conflict resolution -> freeze transaction -> immutable snapshot
```

## 9. 状态所有权与事务边界

- 运行中状态由单次 Use Case/Job 持有。
- 冻结后的输入与证据由不可变账本持有。

## 10. 公共契约与兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。

## 11. 风险与禁止事项

- 任何模型指纹变化立即停止。
- 不得重新计算或“优化”固定概率。
- 不得用当前数据填补历史 cutoff。

## 本阶段 docs 实施记录目录

本阶段使用固定目录：

```text
docs/modular-rewrite/R08-prediction-p4-orchestration/
├─ README.md
├─ R08-01-match-prediction-input-builder.md
├─ R08-02-historical-feature-reader.md
├─ R08-03-readiness-audit.md
├─ R08-04-deterministic-input-manifest.md
├─ R08-05-route-and-model-request.md
├─ R08-06-model-execution-adapter.md
├─ R08-07-run-persistence.md
├─ R08-08-p4-evidence-ledger.md
├─ R08-09-fact-pipeline.md
├─ R08-10-horizon-orchestration.md
├─ R08-11-workbench-reads.md
├─ R08-12-freeze-transaction.md
└─ R08-stage-completion.md
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
11. 每个节点完成时必须创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/<task-record>.md` 并更新阶段 `README.md`；阶段完成时必须创建 `R08-stage-completion.md`。缺少记录不得标记为 `DONE`。

## 原计划阶段摘要（保留用于追溯）

## R8：Prediction / P4 外围链路重写

Atomic Tasks：

- R8-01 match prediction input builder
- R8-02 historical feature reader
- R8-03 readiness audit
- R8-04 deterministic input manifest
- R8-05 route and model request
- R8-06 model execution adapter
- R8-07 run persistence
- R8-08 P4 evidence ledger
- R8-09 fact pipeline
- R8-10 horizon orchestration
- R8-11 workbench reads
- R8-12 freeze transaction

每项必须运行：

- P4 contract。
- P4 engine。
- P4 persistence。
- P4 fact pipeline。
- P4 orchestration。
- P4 workbench。
- P4/P7 model family time window。
- 模型文件指纹。

禁止：

- 为方便重写调整模型参数。
- 重新解释 P4.4。
- 重新计算固定概率。
- 跳过历史 cutoff。

---

# Atomic Tasks

## R8-01 Match Prediction Input Builder

状态：`DONE`（精确 Windows run `36881256338` 通过，见 01 完成记录）

### 1. 目标

- 完成 Match Prediction Input Builder 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- 原模板列出的 Application 根文件已在 R3 迁出；不得重建旧 owner。
- 实际来源：`crates/application/src/use_cases/prediction/execute_prediction_from_match/mod.rs` 的受审计输入构建。
- 原 `PredictionInputPort`、组合适配器与 Persistence `match_prediction.rs` 保持，历史 SQL/贡献读取留 R8-02；评分、manifest、路由、执行和保存留 03～07。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/prediction/build_input/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- `StoredMatchPredictionCommand`、正式/影子模式、现有 PredictionAccess/ModelRegistry；评估结果提供唯一 assessed_at、权限、manifest 与 route identity。

### 6. 输出

- 内部 `PredictionCommand`，携带原输入及审计、原 scope/snapshot/kind、模型选择和显式路由；公共接口保持。

### 7. 允许依赖

- 原 Domain/DTO/Ports、readiness 用例、shared audit/routing；通过原 Port 准备输入，不复制实现。

### 8. 禁止依赖

- 具体数据库/SQL、模型内部/参数、运行保存、额外 Utc::now 或随机请求身份。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 不持有监听器、定时器或共享状态；原调用方控制请求生命周期。单请求顺序执行，权限拒绝、Port/指纹/审计失败直接返回，无新增重试；复用原 assessed_at。

### 13. 兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。

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

- 回退到 R8-01 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-01 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-01-match-prediction-input-builder.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-01-match-prediction-input-builder.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-02 Historical Feature Reader

状态：见阶段索引；精确 `cc2b0fe` / Windows run `36891488571` 全通过，正式完成记录已创建。

### 1. 目标

- 完成 Historical Feature Reader 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- 原模板 Application 根文件已在 R3 迁出。实际来源为 Persistence `team_features.rs` 中球队历史和进球基准读取。
- 上游 `match_prediction.rs::build_prediction_team` 通过原 store 方法调用，Application 沿用 R8-01 builder 与原 PredictionInputPort/组合适配器。
- R6 已收敛的球员贡献/观测/标签查询保留原 catalog owner 和 cutoff；本节点不复制其 SQL 或数学计算。

### 3. 目标文件与目录

```text
crates/persistence-postgres/src/adapters/prediction/historical_features/
crates/persistence-postgres/src/team_features.rs  # 原纯特征投影，无 I/O
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 原 MatchRecord、team_id、主客场方向、data_cutoff_time、PostgresStore pool；固定原 36 候选/4 场范围/12 场上限。

### 6. 输出

- 原 TeamPreMatchFeatures；历史/质量/证据字段、评分、置信度、默认基准保持，SQL/解码错误仍映射原 PersistenceResult。

### 7. 允许依赖

- 原 SQLx/Chrono/Uuid/Domain、现有 store/pool、team_features 纯投影及私有样本类型。

### 8. 禁止依赖

- 模型内部/参数/输出、Application 具体数据库导入、历史写入、额外时钟或重试、新 Port/runner/数据库。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 复用原入参 cutoff，顺序只读两次查询，空历史不查基准，错误即时返回；不新增监听器/共享 State 或 UI 请求机制。原查询不提供事务快照保证，本节点不扩大或虚报一致性保证。

### 13. 兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。

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

- 回退到 R8-02 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-02 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-02-historical-feature-reader.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-02-historical-feature-reader.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-03 Readiness Audit

状态：见阶段索引；精确 `47ba3de` / Windows run `36902069546` 全通过，正式完成记录已创建。

### 1. 目标

- 完成 Readiness Audit 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- 旧 `prediction.rs` 已在 R3 删除，实际来源为 `use_cases/prediction/inspect_match_prediction_readiness/mod.rs` 和 `shared/readiness_checks.rs`。
- Service 和 R8-01 build_input 调用该审计；数据窗口、输入准备和路由由原 Ports 提供。manifest/hash 留 R8-04，routing helper 留 R8-05；不提前迁移 P4 编排或 Persistence SQL。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/prediction/readiness/
  mod.rs           # 模块登记和显式导出
  workflow.rs      # 唯一评估时钟、原只读 Port 顺序与报告组装
  lineups.rs       # 纯阵容/门将/首发上下文检查
  input_quality.rs # 纯历史覆盖/质量检查
  report.rs        # 检查载荷、评分/等级/原因归类
  tests.rs         # 原单测 target 内复用 Probe 的行为测试
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 原 StoredMatchPredictionCommand、原 Ports 和 ModelRegistry；比赛、所选 snapshot、路由、准备输入及原质量证据。

### 6. 输出

- 原 MatchPredictionReadiness、checks/score/level/两种许可、blockers/warnings、cutoff/assessed_at、manifest/hash 和 route_identity，公共字段/错误保持。

### 7. 允许依赖

- 既有 PredictionAccess/Ports、Domain、Application DTO/errors、registry/model-api、shared audit/routing；纯检查仅消费载荷。

### 8. 禁止依赖

- 核心审计不得依赖具体 Persistence/SQL、私有引擎或 UI；不得 predict/save/enqueue、新增后台任务/缓存/重试或校准默认参数。

### 9. 状态所有权

- 所有审计变量为单次 workflow 局部状态，无跨请求共享 State；局部结果在请求结束释放。

### 10. 副作用边界

- 原只读 I/O 集中 workflow.rs，通过已有 Ports；同一 assessed_at 用于 read_match_chain_at 和 prepare_match_input_at，仅 ready_for_model 时准备。不增加读写副作用或改变错误优先级。

### 11. 异常路径

- 窗口/准备 InvalidState、路由 NotFound 继续生成阻断报告，其他 Port 错误原样返回；路由 scope/snapshot/支持性失败进入原模型路由检查。等级优先为 Blocked→ShadowOnly→ReadyWithWarnings→FormalReady，评分不能替代许可。原字符串/权重/metadata/原因顺序保持。

### 12. 并发/异步/生命周期

- 后端为原顺序 await 请求，无新增监听器、定时器或后台生命周期；取消/过期由既有调用方管理，不创建模板请求 ID/空 State。

### 13. 兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变；现有公开 unavailable stub 不冒充私有固定概率 Golden Master。
- 复用原 Probe/单测 target，补齐时钟、门槛、阵容/身份阻断、异常/错误优先级、无模型执行/历史写入及原因去重测试；不创建专项 runner/workflow/target/数据库。
- 八项原 helper、原 async workflow 和报告汇总迁移逐段比对；Windows 为唯一动态验收，真实 PG/Full/私有固定回归及继承删除 trigger 风险仍最终新库待验。

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

- 回退到精确已验证 `cc2b0fe7601efdbad44fa5badcb6507f226698a7`；受控 revert 同步唯一入口/检查/门禁/清单，不手工复制旧文件，不修改历史数据。

### 21. 根 README 摘要记录

- 记录 R8-03 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-03-readiness-audit.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-03-readiness-audit.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-04 Deterministic Input Manifest

状态：见阶段索引；按用户指令开始，须自身精确 Windows CI 和完成记录才能关闭。

### 1. 目标

- 完成 Deterministic Input Manifest 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- 旧 `prediction.rs` 已在 R3 删除，实际来源为 `use_cases/prediction/shared/audit.rs` 的清单构建、运行身份排除、SHA256、受检重建、审计附加与摘要复核。
- 直接调用方为 build_input、readiness/workflow、execute_prediction 及现有单测；原 tests.rs 的两项清单/审计测试一并迁移且保留断言。
- P4 snapshot_projection 的矩阵哈希是独立输出契约，不属于本输入清单责任；routing 仍留 R8-05，Persistence/SQL 不改。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/prediction/input_manifest/
  mod.rs       # 模块登记及显式导出
  canonical.rs # 原清单副本/五字段排除/原 JSON 字节 SHA256
  audit.rs     # 受检重建、审计载荷和摘要复核契约
  tests.rs     # 原单测 target 内两项保留与八项新增测试
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 原输入 JSON、data_quality、MatchRecord、snapshot_type、可选 route_identity；原 PreparedMatchPredictionInput/MatchPredictionReadiness 和调用方提供的完整输入指纹。

### 6. 输出

- 原 manifest JSON、SHA256、小写 hex、input_audit 载荷、可选 PredictionInputAuditSummary 或原 Application 错误；不改变公共字段/格式。

### 7. 允许依赖

- 原 Domain/DTO/Application errors、serde_json、sha2 和 hex；使用已有版本与序列化，不升级依赖或引入新的 canonicalization 算法。

### 8. 禁止依赖

- 纯清单/审计不得依赖具体 Persistence、Ports I/O、模型执行、UI、时钟、随机身份、缓存或后台任务。

### 9. 状态所有权

- canonical 只修改输入副本；attach 仅在校验成功后替换调用者 input_audit 字段。其余无可变共享状态，不创建 State/Coordinator 空壳。

### 10. 副作用边界

- 本能力是纯数据变换/校验，无 I/O；prepare/predict/save 由现有调用方持有。完整输入指纹仍对原 request.input 计算，清单指纹才排除原运行身份。

### 11. 异常路径

- 保持全部原错误类型和字符串，以及缺失清单→缺失指纹→非对象附加、摘要 version→manifest→hash→不匹配的顺序；无审计仍返回 None。复核失败必须先于 predict/save，不静默补造清单或指纹。

### 12. 并发/异步/生命周期

- 函数同步、无生命周期资源；异步调用及取消仍由既有工作流管理，不增加请求 ID/监听器/定时器。

### 13. 兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。
- 只排除根 feature_snapshot_id/input_audit、snapshot.snapshot_id/frozen_at、sources 对象 accessed_at；事实/纳秒 cutoff/阵容贡献/质量/路由保留，数组顺序和 null/缺失继续有意义。
- 保持原 serde_json::to_vec → Sha256 → hex 算法，不修剪身份字段、排序数组、改变浮点或增加时差容差；既有 provider/P4.4 策略不改。
- 六项原函数逐段等价，两项原测试迁移；新增固定公开平台指纹、排除层级、事实敏感、错误/元数据和篡改执行阻断测试进入已有 Application target。没有新 runner/workflow/target/数据库。
- 仅 Windows 动态验收；真实 PG/账本/XLSX/Full/私有固定回归及继承删除 trigger 风险仍最终封包新库待验。

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

- 受控 revert 回精确已验证 `47ba3dea6ca873248728b9846421c009d00f51bc`，同步唯一 owner/调用/测试/门禁/清单；不复制旧实现或修改历史数据。

### 21. 根 README 摘要记录

- 记录 R8-04 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-04-deterministic-input-manifest.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-04-deterministic-input-manifest.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-05 Route 与 Model Request

当前状态与精确门禁结果见 [阶段索引](../modular-rewrite/R08-prediction-p4-orchestration/README.md)。

收尾证据：修复 `2aa99a7` / Windows run `36971419476` 已通过 Application 85/Persistence 141、十项路由请求测试、17 视口及 Windows 交付；见 [05 完成记录](../modular-rewrite/R08-prediction-p4-orchestration/R08-05-route-and-model-request.md)。首轮测试专用 re-export 已限定 cfg(test)，原编排和行为保持。

实际实施：唯一 `route_model_request/{mod,selection,route,request,tests}.rs`，删除 shared/routing.rs；十二原 helper、显式执行上下文及两个请求组装块逐段等价，预览/审计/输入/执行/default 调用唯一 owner。原两测试迁移，八项新测试沿用原 Application target；模型执行/保存留 06/07。Windows CI 单独验收，PG/Full/私有固定回归最终新库待验。

### 1. 目标

- 完成 Route 与 Model Request 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `use_cases/prediction/shared/routing.rs`。
- `execute_prediction/mod.rs` 的显式上下文和实际请求组装。
- `dry_run_default_fixture/mod.rs` 的公开默认请求组装。
- 调用方 preview_route、build_input、readiness/workflow 和 lib 原测试；上述旧根文件早已删除，不能作为本项来源。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/prediction/route_model_request/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 原 PredictionCommand、RoutePreviewCommand 解析值、resolved scope、RouteDecision、RuleRouting、受审计输入和既有 registry。

### 6. 输出

- 规范化选择、UTC/MatchContext、路由身份/校验结果、原 ModelRequest 及既有 ApplicationError；不新增 public DTO。

### 7. 允许依赖

- 原 Domain/model-api、Application DTO/error、registry、chrono/serde_json、既有公开默认外壳函数。

### 8. 禁止依赖

- 具体 Persistence、SQL、UI、私有模型、额外默认参数/算法、新 Port/依赖。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 本目录纯同步函数，无 Port I/O、predict、保存或时钟；原异步读取/执行/保存留在现有用例，不提前迁入 06/07。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 单次调用局部值随请求释放；取消保持原调用方管理，不强制新 State、请求 ID、监听器或重试。

### 13. 兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。

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

- 回退到 R8-05 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-05 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-05-route-and-model-request.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-05-route-and-model-request.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-06 Model Execution Adapter

06 精确提交 `0969331` / Windows run `37752995641` 全 SUCCESS，已收尾为 `DONE`；Application 92/Persistence 141、七项新增测试、17 视口和 Windows 构建/打包/启动均通过。精确状态见 [阶段索引](../modular-rewrite/R08-prediction-p4-orchestration/README.md) 和 [完成记录](../modular-rewrite/R08-prediction-p4-orchestration/R08-06-model-execution-adapter.md)。

### 1. 目标

- 将注册模型查找、支持检查、调用、错误转换及原耗时统计收拢到唯一适配器，保持既有行为。

### 2. 现状与来源

- 旧 prediction.rs 已在 R3 删除；实际来源为 `use_cases/prediction/execute_prediction/mod.rs` 的查找/支持/predict/计时以及 `dry_run_default_fixture/mod.rs` 的查找/predict。
- ModelRegistry 保留原状态与公开 API；P4 冻结经原 Prediction 路径复用，不提前改写 Fact/Workbench/保存。

### 3. 目标文件与目录

```text
crates/application/src/model_registry/prediction_model_adapter.rs
crates/application/src/model_registry/prediction_model_adapter/tests.rs
crates/application/src/use_cases/prediction/execute_prediction/mod.rs
crates/application/src/use_cases/prediction/dry_run_default_fixture/mod.rs
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- model_registry/mod.rs 只声明内部 adapter 并保留原显式公共出口。适配器文件承担同一模型边界责任，测试独立为子模块；按总纲订正不再创建空 execute-model 转发目录。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 原 ModelRegistry、精确 model_id、实际 MatchContext、原 scope CompetitionKind、借用的 ModelRequest。

### 6. 输出

- 原注册 provider Arc、原 ModelOutput，以及成功执行的 i64 毫秒耗时；公共服务接口保持。

### 7. 允许依赖

- 既有 ModelRegistry、ApplicationError/Result、football-model-api、football-domain 与 std Arc/Instant/Duration。

### 8. 禁止依赖

- Persistence/SQL、私有引擎或 stub 直接实现、输入/路由构建、审计、运行保存、概率重写、额外 validate、重试与回退。

### 9. 状态所有权

- 原 ModelRegistry 仍唯一持有模型 Arc；适配器只克隆原句柄并持有局部耗时，不增加缓存或共享状态。

### 10. 副作用边界

- 模型调用集中到 adapter；路由 Port I/O 和正式保存/影子 nil 仍留原执行用例，保存拆分留 R8-07。默认 dry run 不增加 supports/validate/计时。

### 11. 异常路径

- 原查找→supports→请求组装→审计→predict→保存顺序保持；缺失模型原 ID、unsupported 的显示名/原 scope 类型及五类 ModelError 完整提示保持。失败不重试、不写历史。

### 12. 并发/异步/生命周期

- 适配器是原同步模型调用，不新增任务/监听器/定时器/回调；请求及取消生命周期继续由原调用方管理，不机械增加 UI State 或取消协议。

### 13. 兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 登记唯一内部 adapter，保留 ModelRegistry 原出口；不创建没有真实职责的目录或转发层。
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

- 原 Application target 追加 7 项适配器边界测试，预期 92/Persistence 141；须自身 Windows CI 实跑。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

以下动态回归仅在现有 Windows CI 执行；本地可执行 Node 静态检查和同版本 Rustfmt，不运行 Linux/macOS Cargo/客户端验证，不新增专项 runner/workflow/DB。

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

- 回退到 R8-06 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-06 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-06-model-execution-adapter.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-06-model-execution-adapter.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-07 Run Persistence

状态：`DONE`（精确 `58b390a` / Windows run `37796909083` 全 SUCCESS）。详见 [完成记录](../modular-rewrite/R08-prediction-p4-orchestration/R08-07-run-persistence.md)。

### 1. 目标

- 完成 Run Persistence 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- 实际来源为 `crates/persistence-postgres/src/model_runs.rs`：保存事务、输入审计与快照前检、模型明细、历史读取、隐藏事务及原一项 inline 测试。旧 Application `prediction.rs` 在 R3 已删除，不以不存在的文件为来源。
- 上游为 `use_cases/prediction/execute_prediction/mod.rs`、`PredictionExecutionPort` 与 `composition/adapters/prediction.rs`；正式保存/影子 nil 和结果组装保持同一执行用例，不增加仅转发的 Application 目录。
- 历史身份读取复用 R5-06 `adapters/rules/model_run_identity/read`；原共享 mapping/audit owner 不复制。

### 3. 目标文件与目录

```text
crates/persistence-postgres/src/adapters/prediction/runs/
├─ mod.rs
├─ write.rs
├─ input.rs
├─ details.rs
├─ read.rs
└─ visibility.rs
```

### 4. 文件职责边界

- mod 只登记私有模块并导出原 DTO；write 唯一拥有保存事务及完成审计；input 纯前检；details 借用同一事务写模块/比分明细；read 只读原历史投影；visibility 唯一拥有隐藏与审计事务。
- 每个文件只承担上述单一职责；输入前检没有 SQL、时钟或共享状态。
- 业务执行用例通过原 Port 调用 PostgresStore，结果组装不拆成空转发层。

### 5. 输入

- RouteDecision、ModelRequest、ModelOutput、duration_ms；历史 limit；运行 UUID 和可选隐藏原因。

### 6. 输出

- 原保存 UUID、ModelRunListItem 列表、原 read_run JSON、隐藏成功结果和 PersistenceResult 错误；字段、签名及路径保持。

### 7. 允许依赖

- 原 PostgresStore/PgPool、SQLx transaction、Domain/Model API、mapping、audit、R5-06 身份读取、Serde/chrono/UUID；仅使用现有依赖，不增加框架。

### 8. 禁止依赖

- UI/Tauri、Application 内部状态、私有模型实现、Research/P4 账本写入；不重复实现共享审计或输入指纹算法。

### 9. 状态所有权

- 沿用 PostgresStore 的唯一连接池；每次正式保存或隐藏各拥有一个局部事务，不新增 State、缓存、后台任务或自动重试。

### 10. 副作用边界

- 保存前完成原审计/快照前检；runtime 快照复用或创建、run、明细和完成审计在 write 的同一事务提交。details 只借用该事务，不自行开池/提交。隐藏更新及审计在 visibility 同一事务。read 只读。

### 11. 异常路径

- 保持原错误类型、中文提示和检查优先级；任何 SQL/明细/审计错误结束同一事务并向原 Port 映射，执行用例不吞错、不返回成功或重试。

### 12. 并发/异步/生命周期

- 保存前按原顺序生成运行 UUID；每次 async 调用借用原请求/输出，局部 transaction 随成功提交或失败/取消 drop 释放并遵守原 SQLx 回滚行为。快照冲突沿原 ON CONFLICT 和精确查找复用，不增加并发策略；没有 UI 监听器/计时器，不套用 UI 请求状态模板。

### 13. 兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。
- 原四个公开方法签名、DTO 字段、九项生产函数体和 SQL 保持；历史 limit 夹到 1～500，只显示成功且未隐藏记录，顺序/名称 fallback/比分/身份不变；重复隐藏保留首次时间，原记录仍可 read_run。

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

- 回退到 R8-07 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-07 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-07-run-persistence.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-07-run-persistence.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-08 P4 Evidence Ledger

状态：`DONE`（精确修复 `52f23ab` / Windows run `37817443918` 全SUCCESS）。详见 [完成记录](../modular-rewrite/R08-prediction-p4-orchestration/R08-08-p4-evidence-ledger.md)。

### 1. 目标

- 完成 P4 Evidence Ledger 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- 实际来源是 `crates/persistence-postgres/src/p4_records.rs` 的 append_evidence_claim/create_evidence_conflict、声明前检/指纹、版本引用、Row及证据状态投影；旧Application文件在R3已删除。
- 上游为ResearchService/ledger及FactPipeline经原ResearchEvidenceLedgerPort/composition调用；Application不增加空转发层。
- 同文件供Schema/Research/Snapshot使用的幂等锁/键/指纹比较迁入P4 idempotency唯一共享owner，非账本职责原位保持。

### 3. 目标文件与目录

```text
crates/persistence-postgres/src/adapters/p4/
├─ mod.rs
├─ idempotency.rs
└─ evidence_ledger/
   ├─ mod.rs
   ├─ claims.rs
   ├─ conflicts.rs
   ├─ input.rs
   ├─ references.rs
   ├─ row.rs
   └─ tests.rs
```

### 4. 文件职责边界

- claims和conflicts分别拥有一个业务原子事务；input仅纯前检/指纹，references只借用原事务核对版本和冲突引用，row负责声明投影与共享六状态映射，tests复用原单测target。
- idempotency只持有共用事务锁/键/精确指纹判定；mod仅登记/导出，无重复owner。

### 5. 输入

- 原EvidenceClaimDraft/EvidenceConflictDraft，含比赛实体字段、来源/时间/研究和版本引用、幂等键与成员。

### 6. 输出

- 原EvidenceClaimRecord/EvidenceConflictRecord与PersistenceResult错误；签名/字段/Serde/公开路径保持。

### 7. 允许依赖

- 原PostgresStore/PgPool/SQLx Transaction、Domain、audit/sha256_json、Serde/UUID/BTreeSet及现有依赖。

### 8. 禁止依赖

- UI/Tauri、Application内部State、私有模型算法、网络研究执行、FactPipeline/Snapshot/Workbench/Freeze写入；不新增通用框架。

### 9. 状态所有权

- 原PostgresStore连接池保持唯一；每次声明/冲突拥有局部单事务，原同键pg_advisory_xact_lock序列化，不新增缓存/后台任务/重试State。

### 10. 副作用边界

- claims前检→hash→单事务锁→已有精确指纹重试或引用检查→声明→审计；conflicts去重前检→单事务锁→重试或全成员身份核对→头/成员/opened事件/审计。每次新建和重试各沿原出口commit，不在helper自行开池或提交。

### 11. 异常路径

- 保持原错误/中文提示和优先级；不存在或身份/版本不匹配、SQL和审计失败沿原边界返回并释放/回滚事务，不吞错或自动重试。

### 12. 并发/异步/生命周期

- 沿原键命名和局部transaction生命周期，同键锁随提交或失败/drop释放。没有UI监听器/定时器，不套用UI请求模板；取消仍遵循原调用方及SQLx事务行为。

### 13. 兼容要求

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。
- 原来源空字符串检查不额外trim；240字节键限制、精确nanosecond载荷指纹、BTreeSet成员顺序/去重、冲突metadata不入原指纹、声明metadata入指纹及append-only保持。版本/研究/冻结等剩余职责不提前迁移。

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

- 回退到 R8-08 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-08 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-08-p4-evidence-ledger.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-08-p4-evidence-ledger.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-09 Fact Pipeline

状态：`DONE`（精确 `4025781` / Windows `37825126808` 全SUCCESS，详见09完成记录）

### 1. 目标

收敛事实处理编排、输入准备和P4事实记录的唯一职责；保留既有Ports、处理顺序、来源/实体/时间/冲突/路由政策。

### 2. 现状与来源

实际来源是 `crates/application/src/use_cases/research/fact_pipeline/`：原 `mod.rs` 混合公共命令、组合访问trait、artifact注册、主编排和事实准备；其余已有实体/时间/来源/证据/冲突/路由模块，继续复用。旧根 `prediction.rs`、`fact_pipeline.rs`、`p4_persistence.rs`、`p4_workbench.rs` 已在前阶段退出，不再作为来源。

Postgres实际来源为 `crates/persistence-postgres/src/fact_pipeline_records.rs`，混合8个公开方法和9个内部helper，包括来源策略、上下文/候选查询和实体/时间/评估/事件/路由记录。

### 3. 目标文件与目录

- Application既有 `use_cases/research/fact_pipeline/` 新增 `command.rs`、`process.rs`、`prepare.rs`，不创建无调用的 `p4/fact-pipeline` 模板目录。
- Postgres既有P4 adapter新增 `adapters/p4/fact_pipeline/{mod,source_policy,context,candidates,entity_resolution,time_audit,conflicts,routes,fingerprint}.rs`。
- 详细唯一owner及完整文件清单见 [09实施记录](../modular-rewrite/R08-prediction-p4-orchestration/R08-09-fact-pipeline.md)。

### 4. 文件职责边界

Application mod只登记和显式导出；command持有原DTO；process只顺序编排；prepare组合实体/主客队/时间/引用校验并产出分组输入。原计算helper保留既有owner，取消 `use super::*`，不复制公式。来源artifact注册归入既有source_policy。Postgresmod只登记，查询、来源策略和各记录生命周期分开；共享指纹冲突helper只由该目录持有。冲突评估和事件同属conflicts，避免单函数空转发层。

### 5. 输入

既有 `ProcessResearchEvidenceCommand`、研究上下文、联网事实/缺失字段及已验证引用索引；校验response_id、比赛键、精确cutoff和schema版本。

### 6. 输出

既有 `FactPipelineSummary` 与原证据/解析/时间/冲突/路由账本，不改变序列化字段、公开类型路径或PostgresStore签名。

### 7. 允许依赖

Domain、Research Gateway已有DTO、既有ResearchService与FactPipelinePort/ResearchEvidenceLedgerPort/ResearchArtifactPort；Application组合根继续负责具体Postgres接入。Postgres内部复用原pool、hash和审计writer。

### 8. 禁止依赖

Application use case禁止SQLx/PgPool/具体Store、Tauri或UI；不引入私有模型算法、参数、概率默认值或新依赖、工作流、target、DB设施。

### 9. 状态所有权

summary、引用索引、BTreeMap分组是单次调用局部值；无新增共享State、缓存或后台生命周期。持久化记录由原数据库约束唯一持有。

### 10. 副作用边界

Application所有I/O走原Ports；SQL只在P4 adapters。来源策略验证→事务/同key版本锁→幂等读回或插入→同事务审计→提交。其他事实记录沿用原独立写入、冲突读回和精确指纹检查，不虚构整条pipeline原子事务。

### 11. 异常路径

命令无效在context前停止；context身份不一致在记录写入前停止。原事实来源缺失发生在实体/时间写入后，这些既成记录保留；后续证据/路由不执行。Port错误保留kind/message并立即返回，无新增重试或错误兜底。

### 12. 并发/异步/生命周期

保留顺序准备全部事实→BTreeMap有序分组处理→缺失字段处理。等分候选不猜测；主客队filter、历史日期查询、截止纳秒校验保持。独立记录仍依数据库唯一键/ON CONFLICT及原指纹约束收敛，来源策略仍用原事务锁；不增加UI请求ID/取消/销毁模板。

### 13. 兼容要求

命令/DTO/43 Ports/171 Tauri命令/365 Domain类型/300映射/46迁移保持。来源等级、冲突安全唯一赢家、路由版本、幂等键、SQL与提示保持。P4.4 SHADOW_ONLY、P7模型资产和固定概率逻辑不修改；公开保护资产通过不能替代私有Golden Master实跑。

### 14. 实施步骤

1. 从08收尾基线 `6718c996613edc6e4770457ed02fff16f8a95e26` 核对实际入口。
2. 保留原纯职责，提取command/process/prepare并精确显式导出。
3. 将混合Postgres owner按查询/策略/记录生命周期提取，删除旧文件/根登记。
4. 原Application/Persistence单测target新增7+7边界测试；原Stage C/E数据库入口扩读回/幂等/不可变断言，仍ignored。
5. 强化既有Research门禁的owner/依赖/调用顺序/旧实现清理，精确刷新原清单；不新增runner。
6. 完成等价比较、静态门禁、破坏探针与记录，推送自身Windows CI；启动后停止轮询。

### 15. 切换入口

ResearchService、OpenAI Gateway、P4 worker、组合adapter和公开Application入口继续原路径；mod显式指向新process/command。PostgresStore同名方法由P4内的新owner直接实现，不留旧转发。

### 16. 删除清单

删除 `crates/persistence-postgres/src/fact_pipeline_records.rs` 及根登记；移除Application父模块混合实现/通配导入。无整文件移动/重命名，无删除既有测试。

### 17. 最小验证

Research专项、完整architecture、原83源码检查、Rustfmt目标文件、18模型保护资产、171命令、46迁移/18PG静态基线与diff检查；生产函数/SQL/签名/DTO对照。真实Rust编译/单测由09精确Windows CI验证。

### 18. 阶段回归

既有Windows Automated执行 `npm run verify:frontend`、fmt、Clippy、`cargo test --locked --workspace`、17视口、release/MSI/NSIS/启动。不运行Linux/macOS Cargo或客户端；18项PG ignored、真实XLSX、Windows Full及私有Golden Master最终封包新库待验。

### 19. 失败停止条件

保护资产、公开契约或兼容语义变化；本地必需门禁失败；精确Windows失败；与用户未提交修改冲突。失败修复仍限09，不进入10。

### 20. 回退点

用受控revert恢复本项基线 `6718c99`，同时恢复唯一owner、入口、清单、测试和文档；禁止手工复制双实现或修改历史数据。

### 21. 根 README 摘要记录

记录实际拆分、旧文件删除、兼容检查、14新增边界测试及已执行/待执行门禁。

### 22. docs 阶段节点详细记录

创建 [R08-09-fact-pipeline.md](../modular-rewrite/R08-prediction-p4-orchestration/R08-09-fact-pipeline.md)，完整列出A/M/D及无移动、决定/偏差/异常/事务边界、验证与真实风险；更新索引。CI启动保持VERIFYING，10～12BLOCKED；不将预期数或ignored标为PASS。

### 23. 完成标准

唯一新owner和旧实现退出、全部静态与09精确Windows门禁通过、README/任务书/节点/索引一致，才收尾DONE；下一项仍须用户授权。

---

## R8-10 Horizon Orchestration

状态：`DONE`（精确修复 `481bfcb` / Windows `37886029199` 全SUCCESS，详见10完成记录）

### 1. 目标

按实际职责拆分正式时点规划、后台分派和任务持久化，并修复任务首建后入队/绑定失败时重试无法恢复的已确认缺口。

### 2. 现状与来源

实际来源为 `use_cases/prediction/plan_p4_horizons/mod.rs`、`use_cases/p4_orchestration/{mod,process_next,failure}.rs` 和 Postgres 根 `p4_orchestration.rs`。R3已删除旧Application根文件；不创建已过时的Research horizon模板。原planner先创建PLANNED，再入队/绑定；中途失败后重试只返回existing，任务无法继续。

### 3. 目标文件与目录

Application规划保留原目录，拆为 `plan_p4_horizons/{prepare,schedule,queue,process}.rs`。后台分派在原 `use_cases/p4_orchestration/dispatch.rs`，领取/结算仍在process_next。Postgres迁入 `adapters/p4/horizon/{context,input,read,tasks,events,row}.rs`，两个mod只登记/显式导出。

### 4. 文件职责边界

prepare锁定比赛/显式路由/两Schema/29事实；schedule纯计算时点身份及原draft；queue拥有PLANNED恢复、截止判断和原队列绑定；process只编排。dispatch解析任务载荷并委托原Research/Prediction。Postgres context读取规划引用，input纯前检/指纹，read读取任务/事件，tasks拥有事务，events借用事务，row唯一投影。

### 5. 输入

原PlanP4HorizonsCommand：match_id、显式rule_package_id、requested_fact_keys；原BackgroundJob的job_type/payload/id/attempts；原task draft/transition。空事实请求代表全部29项，非空仍须与注册表精确集合相等。

### 6. 输出

原顺序T-24h/T-6h/T-1h三条任务、原队列JSON结果及既有Postgres DTO。T-90m/T-N保留历史读取兼容，不进入正式计划创建。

### 7. 允许依赖

原Domain、PredictionWorkflowPort/JobQueuePort/ResearchArtifactPort/RuleRoutingPort、现有ModelRegistry与Research/PredictionService；SQLx、审计和指纹只在Persistence职责。不增加依赖版本。

### 8. 禁止依赖

Application不得访问具体Postgres/SQLx或私有模型；不得复制Fact Pipeline、Workbench聚合、冻结模型执行和快照事务。Root剩余readiness/冻结查询/路由事实五项仍原位，交后续11/12审查。

### 9. 状态所有权

准备信息、单次捕获时钟和三任务数组只属于本请求；任务/事件由原Postgres持有，后台AtomicBool仍由原Service/worker唯一持有。无新缓存、State、后台线程或前端状态。

### 10. 副作用边界

Application经原Ports创建/入队/迁移；首建和迁移各保持任务+事件+审计共同事务。队列预约与任务绑定仍为两个独立Port操作，不承诺整个三时点或planner原子回滚。

### 11. 异常路径

前检与Port错误即时传播原kind/message，保留前面已完成的时点及部分副作用。已有任务先复核版本/Schema/事实；仅PLANNED可以恢复，未来用原幂等队列键绑定，截止相等/已过转MISSED，其他状态只读返回。队列已存在但绑定失败的恢复仍复用该job；过期时不删除旧job，由原worker终态策略处理。后台达到次数上限才尽力标记合法非终态FAILED，随后原队列fail；fail本身错误仍按原优先级传播。

### 12. 并发/异步/生命周期

保留单次Utc::now、逐时点顺序await、原队列预约/3次尝试与30秒轮询、AtomicBool与数据库断连退出。Postgres首建保留同键事务锁与首次指纹核对，状态迁移保留FOR UPDATE、同状态无写重试、expected冲突及合法迁移。无新增自动重试循环或UI监听器。

### 13. 兼容要求

公开API/DTO/Schema、43 Ports、171命令、配置/日志/UI/原错误保持；模型、P4.4权限、P7、29事实、两版Schema、原cutoff/15分钟lead与grace、键/priority/attempts和0001～0046保持。明确行为修复仅为上述PLANNED恢复；不把该修复称为全行为等价。

### 14. 实施步骤

从09已验源码及文档基线 `7c1ccd34f9d72d13fc5d4b76104cbb248fc2c63a` 开始；核对实际来源与缺口，按职责提取、切换原入口并清理旧实现；复用原Probe和目标补15 Application/6 Persistence测试及原Stage C夹具；扩展原Prediction/Research/Mapping门禁，刷新原清单；核对原函数/SQL/载荷，恢复所有破坏探针；更新四文档与本项记录，推送同分支启动原Windows CI后结束轮询。

### 15. 切换入口

原PredictionService仍调用plan_p4_horizons::execute显式出口；原P4Service调用process_next，后者通过dispatch委托原服务。PostgresStore公开方法路径/签名保持，只有真实实现owner迁移。

### 16. 删除清单

旧planner和跨服务mod中的实现已迁出；Postgres根18项函数/9公开方法迁出，保留后续五函数；无完整文件删除、移动或空转发壳，无双实现。

### 17. 最小验证

本地83源码检查、完整architecture、Rustfmt 1.88的23目标文件、18保护资产/171命令/46迁移及18PG静态契约、差异检查通过。原23 Postgres函数（前检重新内联）及规划准备/身份/draft/未来队列、分派/失败/结算核对等价；六破坏探针拒绝并恢复。详细证据见实施记录。

首轮 `db5fd51` / run `37884818742` 在保留readiness的 `ResearchRunStatus` 缺导入处报E0433，前端/17视口已通过，Rust测试及打包启动未完成。修复只补生产导入、强化原专项依赖检查与刷新Domain使用摘要；8相关生产文件依赖核对、去导入探针/恢复、保留五函数体等价与既有静态门禁通过。不得将修复源码或静态通过记作Windows成功；新head仍须独立实跑。

### 18. 阶段回归

复用既有Windows Automated：完整frontend/5浏览器项与17视口、类型/构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS/启动。预期Application114/Persistence169只是源码预期，须10精确head实跑；无Linux/macOS Cargo/客户端动态验收。真实PG/历史四项/账本/有效XLSX/Windows Full/私有Golden Master仍最终封包新库待验，ignored不计PASS。

### 19. 失败停止条件

保护资产、公开契约、依赖/迁移变化或门禁失败必须修复，10保持VERIFYING；不得继承09 PASS、提前开始11/12或创建R8阶段完成记录。无法取得私有引擎/数据库实际结果时明确保留待验。

### 20. 回退点

受控revert本项至 `7c1ccd3`，同步owner/出口、原测试/门禁/清单及记录；不复制旧文件形成双实现，不改历史库。

### 21. 根 README 摘要记录

记录09精确通过及10实际职责、PLANNED修复、静态PASS/Windows待验、测试源码预期、文件清单链接与11～12BLOCKED。

### 22. docs 阶段节点详细记录

建立 [R08-10-horizon-orchestration.md](../modular-rewrite/R08-prediction-p4-orchestration/R08-10-horizon-orchestration.md)，完整A/M清单，无移动/删除，记录职责、修复偏差、事务/异步/部分副作用边界、验证、异常、回退及Mermaid/Create State；同步索引。Windows启动后停止轮询，精确结果取得前不关闭。

### 23. 完成标准

唯一owner及原入口兼容，明确修复的回归、所有静态及10自身精确Windows门禁通过、文档/实际差异一致才DONE；真实PG等独立待验继续按最终封包约定保留，下一项仍须用户授权。

---

## R8-11 Workbench Reads

状态：`VERIFYING`（10精确修复Windows已全成功并收尾，用户授权“收尾10 开始11”；11等待自身精确Windows，12 BLOCKED）

### 1. 目标

将现有工作台读取的数据库职责从混合人工裁决writer提取到唯一 `adapters/p4/workbench/`，保留两项Application/Port/Store公开契约、原读取顺序与投影。

### 2. 现状与来源

实际来源为 `crates/persistence-postgres/src/p4_workbench.rs` 的 `read_p4_match_workspace` 与 `read_p4_task_workspace`。Application实际已经使用 `use_cases/prediction/read_p4_{match,task}_workspace/mod.rs`，各自仅委托原PredictionWorkflowPort；旧 `application/prediction.rs` 已不存在。人工裁决writer与其四个私有助手、根readiness/routed_facts及快照读取保留原owner。

### 3. 目标文件与目录

PostgreSQL新增 `adapters/p4/workbench/{mod,matches,tasks,research,evidence,conflicts}.rs`。Application在两项原用例补inline测试并复用原Probe，不创建只有转发的research/p4/workbench目录。

### 4. 文件职责边界

mod只显式登记；matches持有比赛/可空赛事与原100任务列表的投影；tasks只汇总现有读取；research读取研究状态/响应/次数/错误/时间；evidence读取当前run的来源与时间证据；conflicts读取run内成员、最新全局事件、run内评估和任务人工裁决。每项SQL与其Row投影共置，不另建通用Mapper。

### 5. 输入

既有 `match_id: Uuid` 或 `task_id: Uuid`；任务记录中的research_run_id/snapshot_id决定可选读取。沿用原实例连接池和上下文，不新增输入、时钟或身份。

### 6. 输出

原P4MatchWorkspace/P4TaskWorkspace及嵌套DTO原样返回；赛事/研究/人工裁决/快照保留Option，缺少研究时证据和冲突为空，NULL人工selected_evidence_ids仍为空数组。

### 7. 允许依赖

Application只用既有PredictionWorkflowPort；composition已有Postgres方法→PortError映射不动。新PG查询用原PostgresStore/PersistenceResult、football_domain、sqlx Row与Uuid，汇总复用原任务/readiness/events/routes/snapshot方法。

### 8. 禁止依赖

不在Application加入SQL/具体持久化；读取owner不创建事务、队列、模型调用、状态迁移、裁决、证据、快照或审计写入，不新增缓存/后台任务/依赖。

### 9. 状态所有权

连接池归原PostgresStore；工作台数据仅由单次请求局部变量拥有，返回DTO。原任务/研究/快照/人工账本状态保持原owner，测试Probe新增字段只在cfg(test)内。

### 10. 副作用边界

生产改动仅把四段原SELECT迁入查询owner；所有原SQL literal逐字保持，绑定值/顺序、fetch_optional/fetch_one/fetch_all、Row字段、错误传播保持。五项人工写入函数逐token保持。

### 11. 异常路径

比赛缺失仍InvalidState“比赛不存在”；任务缺失仍“P4冻结任务不存在”；数据库/Row映射错误仍早停。任务→就绪度→事件→路由→研究→证据→冲突→快照顺序不动，不产生部分成功视图，不增加吞错、重试或回退。

### 12. 并发/异步/生命周期

保留原异步顺序和独立连接池读取；未增加跨查询一致快照事务，因此不承诺各字段来自同一数据库时刻。请求取消、过期UI结果及监听器归原调用方；未创建需释放的后台资源。

### 13. 兼容要求

两Store方法、原Facade/Service/Port/composition/Tauri命令和DTO/serde/Schema、数据格式、配置、错误/日志/UI语义保持；43 Ports/171命令/365类型/300映射、依赖/锁文件、模型与参数、P4.4 SHADOW_ONLY/P7、cutoff与路由及46迁移均保持。

### 14. 实施步骤

先核实10精确Windows成功并五文档收尾；以该同源码文档head为基线，按实际职责提取SQL与汇总，移除旧读实现，补原Application/PG测试及原Prediction门禁，刷新使用清单并复核原逻辑/SQL/写入。

### 15. 切换入口

原public inherent Store方法签名保持，定义分别由matches/tasks唯一持有；既有composition/Service/命令无需改名或新增转发。p4/mod显式登记新workbench，旧根文件只持人工裁决写入。

### 16. 删除清单

移除旧p4_workbench中的两项读取实现及其五个专用Domain导入；没有整文件删除、移动/重命名或旧读空壳。五项writer完整保留，后续Freeze责任未提前改写。

### 17. 最小验证

原Application target增加4测试，覆盖比赛身份/可空赛事/顺序、任务空及丰富视图与进展/终态、两个边界全部六种Port错误且无写入/重试。原ignored Stage C扩真实投影、run/task隔离、NULL来源/时间、排序、人工裁决/全局最新事件、冻结快照、研究元数据与重复/未知ID读取无账本写入。预期Application118（114+4）/Persistence169，须自身Windows实跑，18 broad PG仍ignored。

### 18. 阶段回归

本地83项原前端源码门禁、完整 `npm run verify:architecture`、Prediction/Domain/源码卫生、18保护资产/171命令/46迁移与18 PG静态契约、Rustfmt1.88和diff通过。七项破坏探针均拒绝恢复；四SQL、比赛函数、两Application入口、三个查询投影及重内联任务汇总等价，五writer不变。完整frontend/类型/Vite/17视口、Clippy/workspace tests、Windows release/MSI/NSIS/启动留原Windows CI。

### 19. 失败停止条件

保护资产/公开契约/门禁失败必须修复；11保持VERIFYING，不继承10 PASS，不自动开始12或创建R8阶段完成记录。私有Golden Master或真实PG未执行继续明确待验。

### 20. 回退点

本项基线 `2770adca714fa471e3d801ecd3f541f0f6a05db4`（10已验源码481bfcb的纯文档收尾）。受控revert本节点提交，同步唯一owner、module登记、原测试/清单/门禁与文档；不手工复制双实现或变更历史库。

### 21. 根 README 摘要记录

根README记录实际读取职责和契约保持、7A/14M且无整文件移动/删除、原测试与静态结果及Windows待验；完整清单和风险由节点记录提供。

### 22. docs 阶段节点详细记录

创建 [R08-11-workbench-reads.md](../modular-rewrite/R08-prediction-p4-orchestration/R08-11-workbench-reads.md)，列全A/M/D、原读取与写入边界、测试/探针、真实PG延期、夹具订正、清单/插件与回退；更新索引。11 VERIFYING，12 BLOCKED。

### 23. 完成标准

职责唯一、旧读实现退出且文档一致；自身精确Windows全链路成功后才能补验收证据并关闭11。真实PG/历史四项/账本/XLSX/Windows Full/私有固定回归与继承删除风险继续最终封包新库待验。

---

## R8-12 Freeze Transaction

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Freeze Transaction 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/application/src/prediction.rs`。
- `p4_orchestration.rs`、`fact_pipeline.rs`、`p4_persistence.rs`、`p4_workbench.rs`。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/research/p4/freeze/
crates/persistence-postgres/src/adapters/p4/freeze_transaction/
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

- P4.4 保持 SHADOW_ONLY。
- P7 固定 lambda、概率、矩阵和 top scoreline 回归一致。
- 历史 cutoff、输入指纹、路由和 schema 不变。

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

- 回退到 R8-12 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-12 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-12-freeze-transaction.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R08-12-freeze-transaction.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

# 阶段级验证矩阵

| 验证层级 | 必须执行 | 通过条件 |
|---|---|---|
| 模型保护 | verify protected assets | SHA-256 一致 |
| P4 contract | 全部 contract scripts/tests | 通过 |
| P4 engine | Golden Master | 一致 |
| P7 | fixed regression | 一致 |
| 时间 | cutoff/family window | 无越界 |
| 账本 | freeze/run integration | 不可变 |

# 阶段出口门禁

- `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` 已完整索引全部节点记录。
- `R08-stage-completion.md` 已创建并确认本阶段真实变更、验证、限制和回退点。
- 模型外围新链路接管。
- 所有模型保护与固定回归一致。
- R9 可进入 READY。

# 阶段提交与回退

- 阶段内每个可独立验收的任务保留原子提交；阶段完成提交建议为 `rewrite: complete R8 prediction-p4-orchestration`。
- 只允许回退到最近一个通过全部门禁的提交。
- 不得通过保留双实现代替可回退提交。

# 阶段完成记录要求

本阶段所有 Atomic Task 完成并通过阶段回归后，创建：

```text
docs/modular-rewrite/R08-prediction-p4-orchestration/R08-stage-completion.md
```

必须使用以下结构：

```text
# R08 阶段完成记录

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
