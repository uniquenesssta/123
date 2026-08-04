# R2：Domain 完整重写——独立执行任务书

> 文档编号：`R02`  
> 前置阶段：`R1`  
> 后续阶段：`R3`  
> 本文档是唯一执行依据之一；必须与 `00-总体架构与前23节.md` 同时适用。

## 1. 阶段目标

- 将 `crates/domain/src/lib.rs` 中集中定义拆成按业务语义组织的目录。
- 保持 serde、数据库映射和公共语义完全兼容。

## 2. 本阶段解决的实际问题

- Domain 根文件体积过大，多个领域对象共处同一文件。
- 领域对象、历史对象、状态对象和标识对象边界不够清晰。

## 3. 前置输入与进入条件

- R1 架构门禁通过。
- R0 已生成领域类型清单和序列化基线。

## 4. 明确范围

### 4.1 纳入范围

- Competition/Routing。
- Team/Player/Coach/Formation。
- Lineup/Match。
- Prediction/Research 外围对象。
- Review/Postmatch/Analytics/Exchange/AI/Release。
- Domain 根出口。

### 4.2 排除范围

- Application 用例。
- SQL Row。
- Tauri DTO。
- 模型公式与模型内部类型。

## 5. 当前实现来源与扫描重点

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

## 6. 目标目录总览

```text
crates/domain/src/
  competition/
  routing/
  team/
  player/
  coach/
  formation/
  lineup/
  match_record/
  prediction/
  research/
  review/
  postmatch/
  analytics/
  exchange/
  ai_workspace/
  release/
  shared/
  lib.rs
```

## 7. 目录与文件边界规则

- Domain 不依赖 Tauri、SQLx、网络、文件系统或具体模型实现。
- 一个聚合根目录拥有自己的标识、实体、值对象、状态和错误。
- 跨域引用只使用稳定 ID 或明确的只读值对象。

## 8. 数据流、调用流与依赖方向

```text
external DTO -> boundary mapper -> domain constructor
domain entity/value object -> application use case
repository row -> persistence mapper -> domain object
```

## 9. 状态所有权与事务边界

- Domain 对象只保存领域状态，不保存连接、请求、UI 或任务运行状态。
- 历史记录和当前状态使用不同类型，禁止可变复用。

## 10. 公共契约与兼容要求

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

## 11. 风险与禁止事项

- 拆分路径不能造成隐式 serde 变化。
- 禁止用 glob re-export 掩盖无归属类型。

## 本阶段 docs 实施记录目录

本阶段使用固定目录：

```text
docs/modular-rewrite/R02-domain-rewrite/
├─ README.md
├─ R02-01-领域类型-and-契约清单.md
├─ R02-02-competition-and-routing.md
├─ R02-03-team-player-coach-and-formation.md
├─ R02-04-lineup-and-match.md
├─ R02-05-prediction-and-research-外围领域.md
├─ R02-06-review-and-postmatch.md
├─ R02-07-analytics-exchange-ai-and-release.md
├─ R02-08-domain-根出口收敛.md
└─ R02-stage-completion.md
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
11. 每个节点完成时必须创建 `docs/modular-rewrite/R02-domain-rewrite/<task-record>.md` 并更新阶段 `README.md`；阶段完成时必须创建 `R02-stage-completion.md`。缺少记录不得标记为 `DONE`。

## 原计划阶段摘要（保留用于追溯）

## R2：Domain 完整重写

### R2-01 领域清单

将现有 domain 类型逐一登记：

- 类型名；
- 当前路径；
- 目标模块；
- 序列化名称；
- 数据库映射；
- 调用方；
- 是否公共兼容类型。

### R2-02 Competition / Routing

重写：

- CompetitionKind
- Competition/Season/Stage/Round
- RulePackage
- ModelIdentity
- RouteDecision
- CompetitionBinding

验证：

- JSON 序列化一致。
- 数据库读写一致。
- 路由契约一致。

### R2-03 Team / Player / Coach / Formation

重写目录对象和历史对象。

验证：

- 中英文名称；
- 外部 ID；
- 位置；
- 默认战术角色；
- 效力履历；
- 可用性；
- 能力历史；
- 删除引用。

### R2-04 Lineup / Match

重写：

- MatchRecord/Draft
- Lineup
- LineupPair
- LineupChain
- LineupPreset

验证：

- 双方原子性。
- 历史时点。
- 阵容类型和快照。
- 预设应用。

### R2-05 Prediction / Research

重写模型外围领域对象：

- MatchContext
- PredictionRun
- Readiness
- InputAudit
- Evidence
- Conflict
- FreezeTask
- Workspace

不得触碰模型 crate。

### R2-06 Review / Postmatch

重写：

- Match facts
- Review workflow
- Ability candidate
- Settlement
- Evidence scoring
- Provider score
- Monitoring

### R2-07 Analytics / Exchange / AI / Release

重写其余领域对象。

### R2-08 Domain 根出口

- 删除根文件中的领域定义。
- 使用显式 re-export。
- 禁止 glob export。
- 删除无归属类型。

阶段验证：

- domain 单元测试。
- serde snapshot/契约。
- 所有 workspace 编译。
- 模型固定回归。
- PostgreSQL 集成测试。

---

# Atomic Tasks

## R2-01 领域类型与契约清单

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 领域类型与契约清单 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

### 3. 目标文件与目录

```text
architecture/domain-type-inventory.json
crates/domain/tests/serde_contracts/
```

### 4. 文件职责边界

- 登记每个类型、当前路径、目标路径、serde 名称、数据库映射、调用方和兼容级别。

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

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

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

- 回退到 R2-01 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R2-01 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R02-domain-rewrite/R02-01-领域类型-and-契约清单.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R02-domain-rewrite/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R02-01-领域类型-and-契约清单.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R2-02 Competition 与 Routing

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Competition 与 Routing 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

### 3. 目标文件与目录

```text
crates/domain/src/competition/mod.rs
crates/domain/src/competition/competition.rs
crates/domain/src/competition/season.rs
crates/domain/src/competition/stage.rs
crates/domain/src/competition/round.rs
crates/domain/src/competition/rule_package.rs
crates/domain/src/routing/mod.rs
crates/domain/src/routing/model_identity.rs
crates/domain/src/routing/route_decision.rs
crates/domain/src/routing/competition_binding.rs
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

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

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

- 回退到 R2-02 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R2-02 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R02-domain-rewrite/R02-02-competition-and-routing.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R02-domain-rewrite/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R02-02-competition-and-routing.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R2-03 Team、Player、Coach 与 Formation

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Team、Player、Coach 与 Formation 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

### 3. 目标文件与目录

```text
crates/domain/src/team/
crates/domain/src/player/
crates/domain/src/coach/
crates/domain/src/formation/
```

### 4. 文件职责边界

- 实体、稳定 ID、名称、Profile、历史区间、位置、角色、可用性、能力和标签分目录。

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

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

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

- 回退到 R2-03 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R2-03 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R02-domain-rewrite/R02-03-team-player-coach-and-formation.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R02-domain-rewrite/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R02-03-team-player-coach-and-formation.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R2-04 Lineup 与 Match

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Lineup 与 Match 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

### 3. 目标文件与目录

```text
crates/domain/src/lineup/
crates/domain/src/match_record/
```

### 4. 文件职责边界

- 比赛草稿与已保存记录分离。
- 阵容侧、阵容对、链路和预设分离。

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

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

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

- 回退到 R2-04 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R2-04 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R02-domain-rewrite/R02-04-lineup-and-match.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R02-domain-rewrite/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R02-04-lineup-and-match.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R2-05 Prediction 与 Research 外围领域

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Prediction 与 Research 外围领域 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

### 3. 目标文件与目录

```text
crates/domain/src/prediction/
crates/domain/src/research/
```

### 4. 文件职责边界

- 只定义模型外围上下文、运行、审计、证据、冲突、冻结任务和工作区。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 不得依赖 `model-p4` 或 `model-p7`。
- 不得复制模型内部概率类型或公式。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

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

- 回退到 R2-05 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R2-05 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R02-domain-rewrite/R02-05-prediction-and-research-外围领域.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R02-domain-rewrite/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R02-05-prediction-and-research-外围领域.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R2-06 Review 与 Postmatch

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Review 与 Postmatch 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

### 3. 目标文件与目录

```text
crates/domain/src/review/
crates/domain/src/postmatch/
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

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

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

- 回退到 R2-06 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R2-06 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R02-domain-rewrite/R02-06-review-and-postmatch.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R02-domain-rewrite/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R02-06-review-and-postmatch.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R2-07 Analytics、Exchange、AI 与 Release

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Analytics、Exchange、AI 与 Release 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

### 3. 目标文件与目录

```text
crates/domain/src/analytics/
crates/domain/src/exchange/
crates/domain/src/ai_workspace/
crates/domain/src/release/
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

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

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

- 回退到 R2-07 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R2-07 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R02-domain-rewrite/R02-07-analytics-exchange-ai-and-release.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R02-domain-rewrite/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R02-07-analytics-exchange-ai-and-release.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R2-08 Domain 根出口收敛

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Domain 根出口收敛 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/domain/src/lib.rs`。
- `crates/domain/src/*` 已有子模块。
- 调用方中的直接根路径导入。

### 3. 目标文件与目录

```text
crates/domain/src/lib.rs
```

### 4. 文件职责边界

- 根文件只声明模块并显式 re-export 公共类型。

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

- serde 字段名、枚举表示、默认值和 optional 语义不变。
- 数据库可读取的历史 JSON 必须继续反序列化。
- P4/P7 输入输出类型和 schema 不变。

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

- 删除根文件内所有实体、枚举、构造器和业务逻辑。
- 删除无调用方且无业务归属的类型。

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

- 回退到 R2-08 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R2-08 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R02-domain-rewrite/R02-08-domain-根出口收敛.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R02-domain-rewrite/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R02-08-domain-根出口收敛.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

# 阶段级验证矩阵

| 验证层级 | 必须执行 | 通过条件 |
|---|---|---|
| Serde | domain serde snapshot/round-trip | 旧样本字节语义兼容 |
| 编译 | cargo test -p domain | 全部通过 |
| 数据库 | PostgreSQL integration | 历史行可映射 |
| 模型 | P4/P7 固定回归 | 完全一致 |

# 阶段出口门禁

- `docs/modular-rewrite/R02-domain-rewrite/README.md` 已完整索引全部节点记录。
- `R02-stage-completion.md` 已创建并确认本阶段真实变更、验证、限制和回退点。
- Domain 根文件不再承载领域定义。
- 每个类型有唯一归属目录。
- 所有历史 JSON、数据库映射和模型回归通过。

# 阶段提交与回退

- 阶段内每个可独立验收的任务保留原子提交；阶段完成提交建议为 `rewrite: complete R2 domain-rewrite`。
- 只允许回退到最近一个通过全部门禁的提交。
- 不得通过保留双实现代替可回退提交。

# 阶段完成记录要求

本阶段所有 Atomic Task 完成并通过阶段回归后，创建：

```text
docs/modular-rewrite/R02-domain-rewrite/R02-stage-completion.md
```

必须使用以下结构：

```text
# R02 阶段完成记录

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
