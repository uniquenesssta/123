# R8：Prediction 与 P4 外围链路重写——独立执行任务书

> 文档编号：`R08`  
> 前置阶段：`R7`  
> 后续阶段：`R9`  
> 本文档是唯一执行依据之一；必须与 `00-总体架构与前23节.md` 同时适用。

## 当前执行订正（2026-10-08）

用户已启动唯一 R8 分支 `rewrite/r8-prediction-p4-orchestration`，起点 `90680bf945fbb0d1c191937c2d9e90c2c916fb00`。R7-01～15 及 Windows Automated 已完成；最终代码 `a928c8b` / run `36871154039` 通过。动态 PG、历史四项/账本、有效 XLSX、Windows Full 及模型历史删除风险仍最终封包新库待验，不继承为 PASS。

适用总纲顶部的执行订正：仅 Windows 动态验证，沿用原单测/contract、既有 Windows runner/workflow/数据库入口；不新增专项或持续回归体系。公开仓库没有私有 P4/P7 引擎、参数和 Golden Master；模型保护资产可验证，公共 unavailable stub 的通过不能冒充私有固定概率实跑。目录模板按实际职责及 Rust 模块命名调整；顺序执行的后端 use case 无新增 UI 生命周期，不强制新增 State、请求 ID 或空出口。

R8-01～06 已取得各自精确 Windows CI（`cba72fd` / `36881256338`、`cc2b0fe` / `36891488571`、`47ba3de` / `36902069546`、`59c5679` / `36966815323`、修复 `2aa99a7` / `36971419476`、`0969331` / `37752995641`）及正式完成记录。07 已按用户指令实施、VERIFYING；各项精确当前状态只由 [阶段索引](../modular-rewrite/R08-prediction-p4-orchestration/README.md) 维护。07 及后续必须取得自身 Windows CI，不继承前项 PASS。

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

状态：`VERIFYING`（已按用户“开始07”实施；本项精确 Windows CI 待验，08～12 BLOCKED）。详见 [实施记录](../modular-rewrite/R08-prediction-p4-orchestration/R08-07-run-persistence.md)。

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

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 P4 Evidence Ledger 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/application/src/prediction.rs`。
- `p4_orchestration.rs`、`fact_pipeline.rs`、`p4_persistence.rs`、`p4_workbench.rs`。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/persistence-postgres/src/adapters/p4/evidence_ledger/
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

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Fact Pipeline 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/application/src/prediction.rs`。
- `p4_orchestration.rs`、`fact_pipeline.rs`、`p4_persistence.rs`、`p4_workbench.rs`。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/research/p4/fact-pipeline/
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

- 回退到 R8-09 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-09 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-09-fact-pipeline.md`。
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
- `R08-09-fact-pipeline.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-10 Horizon Orchestration

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Horizon Orchestration 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/application/src/prediction.rs`。
- `p4_orchestration.rs`、`fact_pipeline.rs`、`p4_persistence.rs`、`p4_workbench.rs`。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/research/p4/horizon-orchestration/
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

- 回退到 R8-10 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-10 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-10-horizon-orchestration.md`。
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
- `R08-10-horizon-orchestration.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R8-11 Workbench Reads

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Workbench Reads 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/application/src/prediction.rs`。
- `p4_orchestration.rs`、`fact_pipeline.rs`、`p4_persistence.rs`、`p4_workbench.rs`。
- 相关 persistence modules。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/research/p4/workbench/
crates/persistence-postgres/src/adapters/p4/workbench/
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

- 回退到 R8-11 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R8-11 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-11-workbench-reads.md`。
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
- `R08-11-workbench-reads.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

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
