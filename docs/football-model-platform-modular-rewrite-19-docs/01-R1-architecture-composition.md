# R1：架构契约与空壳组合根——独立执行任务书

> 文档编号：`R01`  
> 前置阶段：`00 文档中的 R0 基线冻结`  
> 后续阶段：`R2`  
> 本文档是唯一执行依据之一；必须与 `00-总体架构与前23节.md` 同时适用。

## 1. 阶段目标

- 建立全仓机器可验证的模块边界。
- 创建浏览器、Tauri、Application 三个组合根，但不迁移业务实现。
- 让后续阶段有稳定依赖方向和注册入口。

## 2. 本阶段解决的实际问题

- 现有入口与组合逻辑分散，`src/main.ts` 和 `src-tauri/src/lib.rs` 容易继续吸收业务职责。
- 当前依赖边界主要依赖人工约定，缺少持续门禁。

## 3. 前置输入与进入条件

- R0 已记录工作区、HEAD、命令契约、迁移指纹、模型保护指纹和前后端基线。
- 不存在未说明的基线失败。

## 4. 明确范围

### 4.1 纳入范围

- architecture 清单与验证脚本。
- 浏览器 bootstrap。
- Tauri bootstrap/state/command registry。
- Application composition/service/model registry 空壳。

### 4.2 排除范围

- 任何具体 Feature 重写。
- Domain 类型迁移。
- 数据库 Repository 重写。
- 模型代码修改。

## 5. 当前实现来源与扫描重点

- `src/main.ts` 启动、注册和全局初始化段。
- `src-tauri/src/lib.rs` setup、manage、generate_handler 注册段。
- `src-tauri/src/commands.rs` 与 commands 子模块出口。
- `crates/application/src/lib.rs` 中 ApplicationService、ModelRegistry 和活动数据库状态。

## 6. 目标目录总览

```text
architecture/
  module-boundaries.json
  state-ownership.json
  command-contract.json
scripts/architecture/
  verifyModuleBoundaries.mjs
  verifyStateOwnership.mjs
src/bootstrap/
  main.ts
  startApplication.ts
  createApplication.ts
  registerApplicationModules.ts
src-tauri/src/
  bootstrap/
    mod.rs
    application.rs
    state.rs
    command_registry.rs
    error.rs
crates/application/src/
  composition/
    mod.rs
    application_composition.rs
    port_registry.rs
  service/
    mod.rs
    application_service.rs
  model_registry/
    mod.rs
    model_registry.rs
```

## 7. 目录与文件边界规则

- 组合根可以依赖具体实现，业务模块不得反向依赖组合根。
- `index.ts`/`mod.rs` 只显式导出。
- 命令注册只登记函数，不实现命令。
- ApplicationService 只作为兼容门面，不拥有具体业务流程。

## 8. 数据流、调用流与依赖方向

```text
browser entry -> bootstrap -> app composition -> feature registration
Tauri Builder -> bootstrap::application -> state injection -> command registry
Tauri command -> ApplicationService facade -> future domain service
Application composition -> ports + model registry
```

## 9. 状态所有权与事务边界

- 浏览器应用生命周期由 bootstrap 创建的 ApplicationHandle 唯一持有。
- Tauri 全局状态由 bootstrap/state 统一构造。
- 模型注册由 application/model_registry 唯一持有。

## 10. 公共契约与兼容要求

- 现有 Tauri 命令名称、参数和返回类型不变。
- 现有应用启动顺序、窗口行为和默认模型注册不变。

## 11. 风险与禁止事项

- 空壳组合根不得成为新的万能容器。
- 不得为了消除编译错误把业务逻辑临时搬入 bootstrap。

## 本阶段 docs 实施记录目录

本阶段使用固定目录：

```text
docs/modular-rewrite/R01-architecture-composition/
├─ README.md
├─ R01-01-模块边界契约.md
├─ R01-02-边界验证脚本.md
├─ R01-03-浏览器组合根.md
├─ R01-04-tauri-组合根.md
├─ R01-05-application-组合根.md
└─ R01-stage-completion.md
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
11. 每个节点完成时必须创建 `docs/modular-rewrite/R01-architecture-composition/<task-record>.md` 并更新阶段 `README.md`；阶段完成时必须创建 `R01-stage-completion.md`。缺少记录不得标记为 `DONE`。

## 原计划阶段摘要（保留用于追溯）

## R1：架构契约与空壳组合根

### R1-01 模块边界契约

创建 `architecture/module-boundaries.json`：

- 前端 feature；
- Rust crate；
- application port；
- persistence adapter；
- Tauri command；
- 允许依赖；
- 禁止依赖；
- 状态所有者。

### R1-02 边界验证脚本

验证：

- 前端 feature 不能直接互相导入内部文件。
- `@tauri-apps/api/core` 只能出现在 platform/tauri。
- SQLx 只能出现在 persistence-postgres 和迁移。
- Tauri 只能出现在 src-tauri。
- model-p4/model-p7 不能被 UI/Tauri/persistence 直接导入。
- application 不依赖 persistence-postgres。
- domain 不依赖基础设施。

### R1-03 浏览器组合根

创建：

- `src/bootstrap/main.ts`
- `createApplication.ts`
- `registerFeatures.ts`
- router/lifecycle 基础模块。

先只迁移现有启动行为，不迁移 feature。

### R1-04 Tauri 组合根

创建：

- `bootstrap.rs`
- `state.rs`
- `command_registry.rs`
- `error.rs`

保持命令不变。

### R1-05 Application 组合根

创建：

- `composition.rs`
- `service.rs`
- `model_registry.rs`
- port 基础结构。

此时可以有适配包装，但阶段结束前必须确认没有改变行为。

阶段验证：

- 架构门禁通过。
- 前端 build。
- Rust fmt/clippy/test。
- 命令契约通过。
- 模型指纹通过。

---

# Atomic Tasks

## R1-01 模块边界契约

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 模块边界契约 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `src/main.ts` 启动、注册和全局初始化段。
- `src-tauri/src/lib.rs` setup、manage、generate_handler 注册段。
- `src-tauri/src/commands.rs` 与 commands 子模块出口。
- `crates/application/src/lib.rs` 中 ApplicationService、ModelRegistry 和活动数据库状态。

### 3. 目标文件与目录

```text
architecture/module-boundaries.json
architecture/state-ownership.json
```

### 4. 文件职责边界

- 记录每个前端 Feature、Rust crate、Application Port、Persistence Adapter、Tauri Command 的允许/禁止依赖。
- 记录每类共享状态的唯一所有者。

### 5. 输入

- 无。

### 6. 输出

- 可机器读取的架构边界和状态所有权清单。

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

- 现有 Tauri 命令名称、参数和返回类型不变。
- 现有应用启动顺序、窗口行为和默认模型注册不变。

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

- 删除重复或过时的人工边界说明；保留历史文档时标记为 superseded。

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

- 回退到 R1-01 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R1-01 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R01-architecture-composition/R01-01-模块边界契约.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R01-architecture-composition/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R01-01-模块边界契约.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R1-02 边界验证脚本

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 边界验证脚本 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `src/main.ts` 启动、注册和全局初始化段。
- `src-tauri/src/lib.rs` setup、manage、generate_handler 注册段。
- `src-tauri/src/commands.rs` 与 commands 子模块出口。
- `crates/application/src/lib.rs` 中 ApplicationService、ModelRegistry 和活动数据库状态。

### 3. 目标文件与目录

```text
scripts/architecture/verifyModuleBoundaries.mjs
scripts/architecture/verifyStateOwnership.mjs
scripts/architecture/verifyProtectedImports.mjs
```

### 4. 文件职责边界

- 解析 TypeScript 与 Rust 依赖。
- 验证平台 API、SQLx、Tauri、模型 crate 的导入范围。

### 5. 输入

- 无。

### 6. 输出

- 非零退出码的 CI/本地门禁。

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

- 现有 Tauri 命令名称、参数和返回类型不变。
- 现有应用启动顺序、窗口行为和默认模型注册不变。

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

- 回退到 R1-02 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R1-02 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R01-architecture-composition/R01-02-边界验证脚本.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R01-architecture-composition/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R01-02-边界验证脚本.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R1-03 浏览器组合根

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 浏览器组合根 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `src/main.ts` 启动、注册和全局初始化段。
- `src-tauri/src/lib.rs` setup、manage、generate_handler 注册段。
- `src-tauri/src/commands.rs` 与 commands 子模块出口。
- `crates/application/src/lib.rs` 中 ApplicationService、ModelRegistry 和活动数据库状态。

### 3. 目标文件与目录

```text
src/bootstrap/main.ts
src/bootstrap/startApplication.ts
src/bootstrap/createApplication.ts
src/bootstrap/registerApplicationModules.ts
src/bootstrap/applicationHandle.ts
```

### 4. 文件职责边界

- 启动入口。
- 创建应用对象。
- 注册模块。
- 集中顶层失败处理。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- ApplicationHandle 是浏览器生命周期唯一所有者。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 现有 Tauri 命令名称、参数和返回类型不变。
- 现有应用启动顺序、窗口行为和默认模型注册不变。

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

- 回退到 R1-03 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R1-03 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R01-architecture-composition/R01-03-浏览器组合根.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R01-architecture-composition/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R01-03-浏览器组合根.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R1-04 Tauri 组合根

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Tauri 组合根 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `src/main.ts` 启动、注册和全局初始化段。
- `src-tauri/src/lib.rs` setup、manage、generate_handler 注册段。
- `src-tauri/src/commands.rs` 与 commands 子模块出口。
- `crates/application/src/lib.rs` 中 ApplicationService、ModelRegistry 和活动数据库状态。

### 3. 目标文件与目录

```text
src-tauri/src/bootstrap/mod.rs
src-tauri/src/bootstrap/application.rs
src-tauri/src/bootstrap/state.rs
src-tauri/src/bootstrap/command_registry.rs
src-tauri/src/bootstrap/error.rs
```

### 4. 文件职责边界

- 构建 Tauri Builder。
- 注入状态。
- 注册命令。
- 映射启动失败。

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

- 现有 Tauri 命令名称、参数和返回类型不变。
- 现有应用启动顺序、窗口行为和默认模型注册不变。

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

- 回退到 R1-04 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R1-04 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R01-architecture-composition/R01-04-tauri-组合根.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R01-architecture-composition/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R01-04-tauri-组合根.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R1-05 Application 组合根

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Application 组合根 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `src/main.ts` 启动、注册和全局初始化段。
- `src-tauri/src/lib.rs` setup、manage、generate_handler 注册段。
- `src-tauri/src/commands.rs` 与 commands 子模块出口。
- `crates/application/src/lib.rs` 中 ApplicationService、ModelRegistry 和活动数据库状态。

### 3. 目标文件与目录

```text
crates/application/src/composition/mod.rs
crates/application/src/composition/application_composition.rs
crates/application/src/composition/port_registry.rs
crates/application/src/service/mod.rs
crates/application/src/service/application_service.rs
crates/application/src/model_registry/mod.rs
crates/application/src/model_registry/model_registry.rs
```

### 4. 文件职责边界

- 构造应用服务与端口。
- 保存兼容门面。
- 注册模型适配器但不修改模型。

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

- 现有 Tauri 命令名称、参数和返回类型不变。
- 现有应用启动顺序、窗口行为和默认模型注册不变。

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

- 回退到 R1-05 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R1-05 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R01-architecture-composition/R01-05-application-组合根.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R01-architecture-composition/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R01-05-application-组合根.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

# 阶段级验证矩阵

| 验证层级 | 必须执行 | 通过条件 |
|---|---|---|
| 架构 | node scripts/architecture/verifyModuleBoundaries.mjs | 所有禁止依赖均被拒绝 |
| 前端 | npm run verify:frontend | 启动行为和构建不变 |
| Rust | cargo test --locked --workspace | 工作区全部通过 |
| 模型 | 保护指纹与固定回归 | 完全一致 |

# 阶段出口门禁

- `docs/modular-rewrite/R01-architecture-composition/README.md` 已完整索引全部节点记录。
- `R01-stage-completion.md` 已创建并确认本阶段真实变更、验证、限制和回退点。
- 三类组合根已建立且不含业务实现。
- 机器边界门禁可在 Windows 与 CI 重复运行。
- R2-01 可进入 READY。

# 阶段提交与回退

- 阶段内每个可独立验收的任务保留原子提交；阶段完成提交建议为 `rewrite: complete R1 architecture-composition`。
- 只允许回退到最近一个通过全部门禁的提交。
- 不得通过保留双实现代替可回退提交。

# 阶段完成记录要求

本阶段所有 Atomic Task 完成并通过阶段回归后，创建：

```text
docs/modular-rewrite/R01-architecture-composition/R01-stage-completion.md
```

必须使用以下结构：

```text
# R01 阶段完成记录

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
