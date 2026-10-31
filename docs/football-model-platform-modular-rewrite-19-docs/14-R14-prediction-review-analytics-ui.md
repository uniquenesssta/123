# R14：推演、复盘与分析业务闭环——执行任务书

> 2026-10-31 接入审计后订正；代码基线：`6e1361210ad4b9fd9f6dd969fad112397e84f6ff`。
> 与[总纲](00-总体架构与前23节.md)共同适用；[审计与全量映射](../modular-rewrite/task-plan-review-20261030.md)。
> 原任务 ID 保留追溯，本阶段改为 8 个可验收交付批次。任务合并不代表功能取消、实现完成或验收通过。

> 最新责任与证据边界见[17页/40项接入处置表](../modular-rewrite/ui-integration-disposition-20261031.md)。本次只订正规划；R12及已完成业务批次不重开，R13-07保持PAUSED。

## 目标、当前依据与范围

复用已稳定的 R8/R10/R11 后端命令与 R12 Platform。前端只消费 capability、就绪审计和结果；正式/影子各有具名工作流，UI 不复制模型算法、概率计算或后端状态机。

当前来源：src/main.ts 中推演/研究/复盘/赛后/分析/发布动作，src/pages/{prediction,p4Workbench,runs,review,analytics,release}.ts，src/app/matchReviewWorkflow.ts 及相应 API/类型。建议归属：`src/features/{prediction,runs,review,postmatch,analytics,release}/`。这是职责定位，不是必须生成的目录清单；实施前读取当前代码，报告与基线的差异。

本阶段交付已批准设计的局部补差、必要业务归属/缺陷修复及缺失验证；不重新设计视觉方向，不从零重建已存在页面。保留搜索/分页/焦点/滚动/草稿/主题/工作区恢复和真实数据能力，不删功能、不改模型、不扩大无关后端重构。设计义务与业务义务分别验收，不能以保留旧HTML代替最新设计接入。

## 执行卡使用前提

本阶段逐批执行下方详细卡，并同时读取[执行约定与验证命令](../modular-rewrite/remaining-execution-contract.md)。下方旧函数行号来自历史 `0b1132bd265adbd7d689c5345986c412fffb3ba4`，只作追溯，不能直接当当前定位；执行时在当前源码按符号追踪实际调用方。最新接入审计基线及前批记录优先，已建立的Feature直接复用，未建立的目标职责按实际需要拆分。

## 状态与依赖

当前全部为 **BLOCKED / 计划未实施**。进入条件为 R13 已按适用出口完成及用户授权；不因本次改写自动 READY。下表顺序是默认交付顺序，必要时在已授权阶段内按真实依赖拆出更小批次并说明原因，不以任务数量或文件数为目标。

| 主交付 ID | 交付批次 | 前置 |
|---|---|---|
| R14-01 | 输入、准入与推演执行 | R13 阶段出口与 R14 启动授权 |
| R14-06 | P4 研究证据与冻结 | R14-01 |
| R14-07 | 运行记录与血缘 | R14-06 |
| R14-08 | 复盘选择、事实与包提交 | R14-07 |
| R14-12 | 赛后结算与监控 | R14-08 |
| R14-13 | 分析查询、长任务与建议包 | R14-12 |
| R14-16 | 能力与参数候选决策 | R14-13 |
| R14-17 | 应用内验收与报告 | R14-16 |

## 执行与验收规则

1. 先检查实际调用图和现有实现。每个批次列明“复用 / 迁移 / 修复 / 补验”；不得因为任务名称包含重构而重写正确算法或已交付基础能力。
2. 状态、工作流、API、传输 DTO 与展示模型分别明确 owner；一个模块内紧密相关的小函数可以共置，复杂独立职责再拆目录。不要求每域建立固定十层目录、一命令一文件、镜像空 CSS/测试目录。
3. 仅在当前证据表明归属或副作用需要调整时迁移本域 main 状态/动作、API/类型消费者及测试；正确且职责明确的实现直接核销重复工程。有迁移时切换唯一调用入口后逐项退出已替代实现。跨域 DTO 选唯一契约 owner，跨域调用走公开接口；不复制类型或引入无期限兼容壳。共享 Foundation 保持唯一实现。
4. 保护现有功能、命令参数/返回/错误、Schema、历史迁移、存储键/版本、Excel/包格式、模型公式/参数/路由/时点和身份血缘。已证实缺陷单列修复并验证，不借兼容保留缺陷。公开不可用模型不能用假数据补成功。
5. 验证跟随每批实现：先执行受影响的行为/竞态/契约检查，再按影响完成现有前端、Rust、Windows 构建/运行门禁。跨层改动包含相应后端检查；不改 Rust 的批次可引用同一后端树已验结果并说明边界。失败必须修复，不能跳过/放宽检查。
6. 静态检查、类型/构建、自动测试、Windows 实际运行、真实外部数据分别记录。可复用证据必须标出提交或受测树摘要、环境、工具、入口和结果；只有代码/依赖/配置/资产均无相关变化才可复用。CI 只按精确提交一次性核对，不轮询。
7. 真实 PG/XLSX/Full 等沿用总纲已批准的最终新库安排。节点可记录“实现及 Automated 完成、真实环境待验”，但不能把它写成 Full PASS；阶段索引显式携带待验项到 R17-06，正式发布必须清零。其余本阶段必需门禁失败不得以延期绕过。
8. 进程 TEMP/TMP 使用项目临时目录并单独提醒配置；继续 Windows/PowerShell。不得新增持续回归框架、runner、workflow、专项数据库、生产依赖或公开私有资产。
9. 分支沿用每阶段一个既定分支；进入阶段同时需要前置出口和用户授权。本次任务书订正不自动恢复或开启开发。后续提交推送沿用用户长期授权，不逐批重复询问。
10. 每批记录实际变更清单、唯一 owner/调用链、验证与未验项、剩余风险和回退点。合并项可共用一份交付记录，但必须逐项列出覆盖证据；不能整组一勾代替业务验收。文档按有效信息组织，不强制填 23 节空模板。

## R14-01 输入、准入与推演执行

### 审计后的实际交付范围

- **直接复用并核销重复建设：** S03比赛/模型/窗口选择、readiness/指纹、正式/影子调用和结果投影；不重写模型或从零创建推演页。
- **本批必须完成：** 将正式/影子状态的Provider警告、设置、准入、输入审计、结果和动作区对应最新设计；许可绑定当前比赛/窗口/指纹，故障查回已有run而非盲目重跑。只迁需要独立owner的状态和副作用。
- **增量验收：** R14-01-D1：同Formal Unchecked、blocked、ready、running、failed、result状态，输入变化立即废弃旧许可/结果；固定底部动作不遮挡。D2：双击/断网结果未知不产生虚假成功或自动重复提交；真实模型按BA-13待验边界。

覆盖原节点：R14-01、R14-02、R14-03、R14-04、R14-05。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-08、BA-13**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 同批交付上下文选择/恢复、readiness/阻断原因/fingerprint、正式运行、影子运行与结果摘要/胜平负/比分矩阵/输入审计。内部保持独立职责，不以合批为理由合成巨型文件。

**业务验收：** 任何影响输入的变化立即使旧准入失效；提交前核对比赛/快照/指纹；重复点击不重复提交；正式/影子结果不能混用；SHADOW_ONLY 不升级；取消/超时区分请求结果未知，先核验再重试；只展示真实输出。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/prediction.ts`、`src/types.ts`、`src/api/client.ts`。代表性现存函数：`selectedStoredPredictionCommand`（main.ts:3870）、`checkPredictionLineupChain`（main.ts:3946）、`calculatePredictionMatch`（main.ts:4024）、`runShadowPredictionMatch`（main.ts:4004）、`predictionViewData`（main.ts:1051）、`showPredictionResult`（main.ts:1353）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`inspectMatchPredictionReadiness`、`readMatchLineupChain`、`executePredictionFromMatch`、`executeShadowPredictionFromMatch`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

prediction/context 拥有比赛/快照/模型选择；readiness 拥有与输入身份绑定的审计；formal/shadow 各有工作流，result只读。

稳定上下文→指纹就绪审计→对应运行入口→后端结果→运行记录。输入变化同步废弃旧许可，提交时再次核对当前输入。

### 原子实施顺序

1. 子步骤A：迁上下文/恢复与输入投影，保留原日期时区、T-N/T-1h等快照语义；冻结现有API与模型选择约束。
2. 子步骤B：迁就绪审计和阻断理由，使所有相关输入变化失效旧结果；用可控延迟测试验证后再继续。
3. 子步骤C：分别迁正式和影子执行，复用当前传输；测试重复点击、取消/未知结果及身份检查。
4. 子步骤D：迁结果投影/矩阵/详情与历史导航，联验A至D；不得前端重算概率或把影子结果变为正式。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R14-01-V1**：审计A成功后改阵容/模型/快照→按钮不能沿用A许可；A迟到审计不使B就绪。
- **R14-01-V2**：连续点击运行→仅一次有效提交；请求超时→显示未知或失败按现有协议，先查运行身份再重试。
- **R14-01-V3**：影子结果到达正式结果视图→仍带真实类型；SHADOW_ONLY被正式入口拒绝。
- **R14-01-V4**：后端字段缺失→保留未知标记，不补概率/比分；切比赛后旧结果不冒充新比赛结果。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-prediction-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-01: verify-prediction-service failed' }
node scripts/verify-model-run-identity.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-01: verify-model-run-identity failed' }
node scripts/verify-match-lineup-chain.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-01: verify-match-lineup-chain failed' }
node scripts/verify-history-scoreline-ui.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-01: verify-history-scoreline-ui failed' }
node scripts/verify-public-model-boundary.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-01: verify-public-model-boundary failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R14-06 P4 研究证据与冻结

### 审计后的实际交付范围

- **直接复用并核销重复建设：** P4工作台、研究任务、证据读取、冻结命令和既有后端；S03共用骨架沿14-01，不另建研究整页。
- **本批必须完成：** 仅接入P4研究/证据/冲突/冻结状态和必要归属；研究身份、输入版本、冻结许可及晚到响应隔离；消除已登记prediction→p4Workbench内部依赖。
- **增量验收：** R14-06-D1：切比赛或输入修订后旧证据/确认不能冻结新对象；人工步骤与Provider unavailable明确。D2：同研究待审/冻结中/失败/已冻结状态对照对应Frame；冻结真实账本/私有回归由BA-02/13逐项承接。

覆盖原节点：R14-06。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-02、BA-08、BA-13、BA-15**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 归属任务目录、证据、冲突与冻结动作，保留来源、时点、引用和审计。

**业务验收：** 冲突未解决时不绕过冻结门禁；同一输入的操作幂等；迟到任务不覆盖当前对象；已冻结快照和不可变账本不可改写。私有模型材料不进入公开仓库。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/p4Workbench.ts`、`src/api/client.ts`、`src/types.ts`。代表性现存函数：`loadP4MatchWorkspace`（main.ts:1947）、`planP4Horizons`（main.ts:4072）、`openP4Task`（main.ts:4089）、`resolveP4Conflict`（main.ts:4101）、`refreshP4Workbench`（main.ts:4095）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`readP4MatchWorkspace`、`readP4TaskWorkspace`、`planP4Horizons`、`resolveP4Conflict`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

prediction/p4-research 持有当前match/task选择及查询，证据/冲突/冻结动作独立；不可变证据和任务终态由后端决定。

比赛→任务→证据及冲突→明确解决→后端冻结许可/结果；仅以DTO投影，不推断已冻结。

### 原子实施顺序

1. 盘点工作台命令和capability/引用/时点字段，确认UI与后端冻结职责。
2. 迁查询及任务选择并绑定match/task ID，再迁冲突解决与刷新。
3. 所有写入继续使用后端状态校验，保留来源/时间审计；冻结显示不能用本地布尔值伪造。
4. 验证重复操作与迟到响应后切换唯一入口，保留私有资产边界。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R14-06-V1**：A任务慢读后选B→不显示A证据为B；未知来源不补造。
- **R14-06-V2**：冲突未解决或许可已失效→冻结被拒绝；失败保留待处理冲突。
- **R14-06-V3**：同一解决动作重试→实际账本幂等且历史不可改；需真实PG证明而非只看按钮。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-research-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-06: verify-research-service failed' }
node scripts/verify-r3-06-p4-freeze.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-06: verify-r3-06-p4-freeze failed' }
node scripts/verify-public-model-boundary.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-06: verify-public-model-boundary failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R14-07 运行记录与血缘

### 审计后的实际交付范围

- **直接复用并核销重复建设：** S05列表、详情、技术追踪及历史隐藏；已有查询/详情/隐藏命令。
- **本批必须完成：** 用相同有数据/缺矩阵/缺身份/empty状态核实是否真有差异，再定点修；确保未知值不补0，隐藏不改底层运行/快照血缘；退出prediction→runs内部依赖。
- **增量验收：** R14-07-D1：六类记录样本逐列核对时间、模型/规则、窗口/审计、比分概率、主平客和操作；无矩阵显示未记录。D2：隐藏前后底层同run可追溯，取消确认零写入；隔离空截图不能核销此项。

覆盖原节点：R14-07。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-08、BA-15**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 归属运行目录/详情、矩阵、输入审计和可见性；复用推演结果只读展示，历史结果不能变成当前执行许可。

**业务验收：** 空值不伪造为零；切换记录正确；隐藏仅改变可见性，不删除血缘或快照；失败不展示上个记录为本次详情。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/runs.ts`、`src/api/client.ts`、`src/types.ts`。代表性现存函数：`requestHideRunHistory`（main.ts:5501）、`showPredictionDetail`（main.ts:1304）、`predictionDetailBody`（main.ts:1251）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`hideModelRunHistory`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

runs 拥有目录/选择/详情加载；结果只读呈现复用prediction的公开只读接口或具名共享投影；隐藏动作单独归属。

run ID→真实存储结果/输入审计→展示；隐藏→后端可见性修改→刷新，不改变原推演血缘。

### 原子实施顺序

1. 核对列表来自Session还是独立查询，选唯一缓存owner，冻结详情/隐藏API与未知值行为。
2. 迁目录选择和详情读取，绑定请求与run ID；复用矩阵展示避免两个结果算法。
3. 迁隐藏确认与刷新，再接回来源导航。
4. 检查历史读取不触发推演，切换入口后撤出重复详情逻辑。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R14-07-V1**：快速选run A/B→A迟到不覆盖B；B读取失败不复用A内容。
- **R14-07-V2**：隐藏run→列表按真实结果更新；数据库中输入快照/引用/报告仍可追溯。
- **R14-07-V3**：历史中空比分/概率→仍未知；查看历史不授予当前比赛执行许可。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-history-scoreline-ui.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-07: verify-history-scoreline-ui failed' }
node scripts/verify-model-run-identity.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-07: verify-model-run-identity failed' }
node scripts/verify-analysis-history-workflow.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-07: verify-analysis-history-workflow failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R14-08 复盘选择、事实与包提交

### 审计后的实际交付范围

- **直接复用并核销重复建设：** S04九步工作流、事实编辑、包导出/预检/确认、结果生成和真实allowed_actions；不重造后端状态机。
- **本批必须完成：** 以同Loaded/事实/预检/确认/结果状态完整挂载核验后补局部差异；保持左侧步骤与内容区滚动边界、包ID/SHA/比赛身份和失败恢复；结算交14-12。
- **增量验收：** R14-08-D1：本轮审计空态不能与Loaded直接判相等，必须补相同对象/步骤；查看锁定步骤不授予执行。D2：预检后换包/比赛→旧确认失效；导出、事实和生成结果分别记成功，部分失败不拼接全成功。

覆盖原节点：R14-08、R14-09、R14-10、R14-11。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-02、BA-07、BA-12、BA-13**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 同批贯通九步 capability、实际赛果/阵容/球员观察、复盘包选择/导出/预览/确认/事实提交/包提交、复盘结果和能力候选展示；各写入动作保持独立职责。

**业务验收：** 每一步按后端许可执行；确认后比赛 ID/包 SHA/输入身份变化须重新预检；事实提交成功而后续失败时可核验恢复，不重复事实；不把生成复盘当作已结算；与 R13 工作簿联验。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/review.ts`、`src/pages/review/matchEventTimelineView.ts`、`src/app/matchReviewWorkflow.ts`。代表性现存函数：`loadSelectedReviewMatch`（main.ts:5895）、`refreshMatchReviewPackageWorkflow`（main.ts:5931）、`previewMatchReviewPackage`（main.ts:5953）、`confirmMatchReviewPackage`（main.ts:5970）、`commitMatchReviewPackageFacts`（main.ts:5998）、`generateMatchReviewFromPackage`（main.ts:6023）、`generateMatchReview`（main.ts:6097）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`readMatchReview`、`readMatchReviewPackageWorkflow`、`chooseExcelImportFile`、`previewMatchReviewPackage`、`confirmMatchReviewPackage`、`commitMatchReviewPackageFacts`、`generateMatchReviewFromPackage`、`generateMatchReview`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

review selection持有比赛身份，workflow投影后端九步capability，facts持有赛果/实际阵容/观察草稿，package持有文件身份/预检/确认；results只读。

match选择→后端许可→真实facts/package预检→明确确认→事实提交→包/复盘生成→结果；每一步不隐含下一步成功。

### 原子实施顺序

1. 子步骤A：迁比赛选择与九步投影，直接复用matchReviewWorkflow helper，不复制状态机。
2. 子步骤B：迁赛果/阵容/球员观察和事件事实编辑，核对实际match ID与输入版本。
3. 子步骤C：迁导出/预览/确认/事实提交/包提交，绑定SHA与对象，故障可恢复；复用R13工作簿接口。
4. 子步骤D：迁复盘结果与候选展示，按后端许可联验全部步骤，特别处理事实已提交后生成失败。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R14-08-V1**：确认包A后替换文件或比赛→确认失效，不能提交旧身份。
- **R14-08-V2**：事实写入成功后生成失败→重进读取当前工作流，只恢复允许的步骤，不能重复事实或谎报完成。
- **R14-08-V3**：许可拒绝、重复点确认、并发修改→后端拒绝准确投影且输入保留；不能本地跳步骤。
- **R14-08-V4**：生成复盘成功→仅表示复盘成功，不自动表示赛后结算/能力候选已应用。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-match-review-package.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-08: verify-match-review-package failed' }
node scripts/verify-match-event-facts.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-08: verify-match-event-facts failed' }
node scripts/verify-review-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-08: verify-review-service failed' }
node scripts/verify-analysis-history-workflow.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-08: verify-analysis-history-workflow failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R14-12 赛后结算与监控

### 审计后的实际交付范围

- **直接复用并核销重复建设：** 现有正式结算、证据评分、供应商和H监控入口/API；复用S04/S12已建立骨架。
- **本批必须完成：** 只负责结算及其监控状态，补对应设计状态而非新建独立整页；校验复盘/正式run/冻结快照关系，分区与幂等明确。
- **增量验收：** R14-12-D1：复盘完成但结算未满足条件→禁止伪造已结算；相同/不同幂等请求分别核验。D2：全局结算数不作为指定H分区许可，结果未知按原身份查回；真实事务/评分由BA-12闭合。

覆盖原节点：R14-12。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-12**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 归属结算目录/详情、证据评分、提供方评分与漂移监控；保留评分和结算审计。

**业务验收：** 重复结算、重复评分的幂等与历史保护；未满足许可不能执行；读失败保留真实失败状态；监控值取后端结果。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/analytics.ts`、`src/pages/review.ts`、`src/types.ts`。代表性现存函数：`inspectPostmatchReadiness`（main.ts:6224）、`settlePostmatchReview`（main.ts:6230）、`refreshPostmatchMonitoring`（main.ts:6257）、`prepareEvidenceDecision`（main.ts:6271）、`submitEvidenceDecision`（main.ts:6282）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`postmatchSettlementReadiness`、`settlePostmatchReview`、`postmatchOverview`、`refreshPostmatchMonitoring`、`decideEvidenceScoringItem`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

postmatch 拥有结算选择/准入、评分草稿及监控查询；review/analytics页面通过其公开接口组合，后端账本唯一。

review ID→结算就绪→明确确认→后端事务→结算读回；证据评分/监控各自查询，不能由页面推算账本终态。

### 原子实施顺序

1. 确定复盘页与分析页的共同赛后调用，建立一个postmatch owner。
2. 迁就绪/结算与确认，再迁证据评分和监控读取。
3. 处理结果未知和重复提交，沿用后端幂等键及错误分类。
4. 联验结算状态与分析刷新，退出两页重复状态。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R14-12-V1**：未就绪或许可过期→无有效结算；允许后结算读回与账本一致。
- **R14-12-V2**：响应丢失后再进入→查询实际结算，不盲目再写；重复请求不重复记分。
- **R14-12-V3**：切换结算A/B→评分草稿不能写入另一结算，慢监控结果不覆盖新对象。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-postmatch-settlement.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-12: verify-postmatch-settlement failed' }
node scripts/verify-postmatch-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-12: verify-postmatch-service failed' }
node scripts/verify-review-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-12: verify-review-service failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R14-13 分析查询、长任务与建议包

### 审计后的实际交付范围

- **直接复用并核销重复建设：** S12分析查询、四类任务、取消/重试、包/建议API和H证据；不重建分析引擎或任务队列。
- **本批必须完成：** 将纵向折叠工作台与批准的顶部指标/横向步骤/内容区逐项协调接入，保留必要信息；分离任务终态、总览快照与精确范围许可；接受建议只进入待审候选。
- **增量验收：** R14-13-D1：同history/analysis/quality状态、同样本范围对照，步骤/指标不因全局快照误解锁。D2：任务取消/失败/切页返回、包冲突与重复决定均有真实生产路径断言；本批骨架供14-16复用。

覆盖原节点：R14-13、R14-14、R14-15。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-12**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 归属总览/模型比较/校准/漂移/数据质量，任务目录/进度/动作，以及分析包导出/响应预览/建议队列。

**业务验收：** 任务 ID/分区/输入上下文绑定；取消或切页后的迟到结果不串线；导入只产生预览和候选；全局快照不授予具体分区许可；失败和未知结果可区分。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/analytics.ts`、`src/controllers/pageLoaders.ts`、`src/api/client.ts`。代表性现存函数：`loadAnalysisCenter`（main.ts:1938）、`enqueueAnalysisTask`（main.ts:6208）、`exportAiAnalysisPackage`（main.ts:6459）、`previewAiAnalysisResponse`（main.ts:6506）、`importAiAnalysisResponse`（main.ts:6523）、`decideAiSuggestion`（main.ts:6538）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`enqueueAnalysisJob`、`chooseAiAnalysisExportFile`、`exportAiAnalysisPackage`、`chooseAiAnalysisResponseFile`、`previewAiAnalysisResponse`、`importAiAnalysisResponse`、`decideAiAnalysisSuggestion`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

analytics 的只读总览、jobs及packages/suggestions分别拥有查询与状态；每个任务/响应包绑定具体分区和对象。

分区输入→后台job→查询真实状态→结果投影；分析包导出→响应预检→候选建议→显式决策，不自动变更生产参数。

### 原子实施顺序

1. 盘点总览字段、任务动作、包与建议接口，区分全局摘要和具体分区授权。
2. 迁总览查询与任务生命周期，使用实际job ID，保留当前取消语义。
3. 迁包预览/导入和建议队列，核对身份/Schema/文件指纹及冲突。
4. 联验跨页恢复和未知任务状态，前端不重新计算校准/漂移/模型比较。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R14-13-V1**：启动分区A任务后切B→A结果留在A，不赋给B；全局ready不代替B许可。
- **R14-13-V2**：取消请求已发送但job仍运行→不能伪造cancelled；终态以实际状态为准。
- **R14-13-V3**：导入响应仅生成预览/建议，重复导入按契约幂等；坏包/跨比赛包拒绝且不写生产候选。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-analytics-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-13: verify-analytics-service failed' }
node scripts/verify-analysis-history-workflow.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-13: verify-analysis-history-workflow failed' }
node scripts/verify-api-workspace-research-control.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-13: verify-api-workspace-research-control failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R14-16 能力与参数候选决策

### 审计后的实际交付范围

- **直接复用并核销重复建设：** 能力候选人工决定、参数候选、影子验证、晋升/回滚API与14-13页面骨架。
- **本批必须完成：** 补S12候选相关状态和当前精确分区/版本/人工确认防护；公开Provider不可用须保留真实阻断，不用固定概率制造通过。
- **增量验收：** R14-16-D1：候选过期/分区变化后旧决定失效；影子结果不等于已晋升。D2：同候选待审/拒绝/影子/晋升/回滚Frame核对动作和状态，真实绑定与回滚读回沿BA-12。

覆盖原节点：R14-16。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-12、BA-13**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 独立管理能力候选、参数候选、影子验证、人工接受、晋升与回滚；不从建议接受自动跳到生产修改。

**业务验收：** 快照/版本/许可与候选绑定；失败不部分晋升；晋升及回滚有明确确认和后端门禁；前端不得修改保护参数或公式，私有引擎固定回归单列。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/analytics.ts`、`src/types.ts`、`contracts/parameter-lifecycle-contract.json`。代表性现存函数：`decideAbilityCandidate`（main.ts:6160）、`generateParameterTuning`（main.ts:6302）、`checkParameterLifecycleReadiness`（main.ts:6345）、`runParameterShadowValidation`（main.ts:6365）、`promoteParameterCandidate`（main.ts:6383）、`rollbackParameterCandidate`（main.ts:6414）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`decideAbilityCandidate`、`readMatchReview`、`generateParameterTuningCandidate`、`listParameterTuningCandidates`、`parameterLifecycleReadiness`、`runParameterShadowValidation`、`promoteParameterCandidate`、`rollbackParameterCandidate`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

analytics/candidates 中能力候选与参数候选分开，影子验证、晋升、回滚具名工作流；后端拥有许可和生产版本。

候选身份/快照→就绪检查→影子验证→人工决策→后端晋升/回滚→历史读回。建议接受不是晋升许可。

### 原子实施顺序

1. 冻结候选状态、影子产物、晋升/回滚请求及版本校验字段，读取现有生命周期契约。
2. 迁只读队列与候选详情，再迁人工接受和影子验证，保持能力候选与参数候选互不混用。
3. 迁晋升与回滚确认，绑定实际版本/快照及后端许可，不构造新参数值。
4. 用失败/冲突验证阻止部分成功，核对审计与历史；保护资产一旦变化停止交付。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R14-16-V1**：验证通过后候选或基线版本变化→旧验证不能用于晋升，必须重新准入。
- **R14-16-V2**：双击晋升或另一操作者已晋升→按后端版本/幂等结果投影，不本地强行成功。
- **R14-16-V3**：回滚失败→不得显示旧版本已恢复；私有模型固定回归未实跑不得标通过。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-parameter-lifecycle.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-16: verify-parameter-lifecycle failed' }
node scripts/verify-analytics-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-16: verify-analytics-service failed' }
node scripts/verify-protected-assets-deterministic.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-16: verify-protected-assets-deterministic failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R14-17 应用内验收与报告

### 审计后的实际交付范围

- **直接复用并核销重复建设：** S11六分区、验收表单、请求/只读报告/历史API，不重复造发布引擎。
- **本批必须完成：** 补顶部四指标与报告状态，移除把公开外壳描述成已具备固定P4运行能力的误导；区分新请求草稿与历史报告。这里是应用内验收，不是R18真实软件发布。
- **增量验收：** R14-17-D1：同未运行/处理中/警告/失败/历史状态检查指标与动作；缺结果用未取得，非零阻断。D2：修改新预算不能改变历史报告，重复/未知请求可查回；完整真实运行不由界面标签代替。

覆盖原节点：R14-17。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-12**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 归属验收表单/执行/历史/详情/分类门禁及报告；消费已有发布服务，程序内报告不直接完成 Git 发布。

**业务验收：** 报告内容与真实结果一致且不可变；公开模型不可用如实显示；历史不可授予本次通过；执行失败不能报告成功；与最终 R18 发布证据分清职责。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/release.ts`、`src/api/client.ts`、`contracts/windows-acceptance-contract.json`。代表性现存函数：`refreshReleaseAcceptance`（main.ts:6808）、`runReleaseAcceptance`（main.ts:6826）、`openReleaseAcceptanceRun`（main.ts:6841）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`listReleaseAcceptanceRuns`、`readReleaseAcceptanceRun`、`runReleaseAcceptance`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

release feature拥有表单草稿、当前验收请求、不可变历史与详情；后端release服务生成报告，不让页面发布Git。

显式验收请求→真实服务→分类检查结果/不可变报告→查询历史；每份报告保留自己的输入和环境标识。

### 原子实施顺序

1. 盘点表单字段、历史列表和报告DTO，对照后端release命令，不新增前端计算门禁。
2. 迁执行、读取和列表状态，并绑定请求/报告ID。
3. 区分完整成功、失败、环境缺失和公开模型不可用，防止历史报告污染新执行。
4. 核对导出/查看只读，接回现有页面及日志；最终软件发布仍由R18负责。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R14-17-V1**：历史成功后发起失败验收→当前结果必须失败，不沿用历史绿色状态。
- **R14-17-V2**：公开引擎不可用→报告真实限制，不填模拟通过。
- **R14-17-V3**：查看/导出已有报告→内容不可改；完成应用内报告不产生Git标签或自动发布。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-release-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-17: verify-release-service failed' }
node scripts/verify-release-acceptance.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-17: verify-release-acceptance failed' }
node scripts/verify-release-readiness.mjs
if ($LASTEXITCODE -ne 0) { throw 'R14-17: verify-release-readiness failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## 原节点去向（完整覆盖）

| 原节点 | 当前主交付 | 处理方式 |
|---|---|---|
| R14-01 Prediction Context | R14-01 | 主交付 |
| R14-02 Readiness 与 Fingerprint | R14-01 | 合并验收项（非 DONE） |
| R14-03 Formal Run | R14-01 | 合并验收项（非 DONE） |
| R14-04 Shadow Run | R14-01 | 合并验收项（非 DONE） |
| R14-05 Prediction Result | R14-01 | 合并验收项（非 DONE） |
| R14-06 P4 Research UI | R14-06 | 主交付 |
| R14-07 Runs | R14-07 | 主交付 |
| R14-08 Review Selection 与 Workflow | R14-08 | 主交付 |
| R14-09 Review Facts | R14-08 | 合并验收项（非 DONE） |
| R14-10 Review Package | R14-08 | 合并验收项（非 DONE） |
| R14-11 Review Results | R14-08 | 合并验收项（非 DONE） |
| R14-12 Postmatch | R14-12 | 主交付 |
| R14-13 Analytics Overview | R14-13 | 主交付 |
| R14-14 Analytics Jobs | R14-13 | 合并验收项（非 DONE） |
| R14-15 Analysis Packages 与 Suggestions | R14-13 | 合并验收项（非 DONE） |
| R14-16 Ability 与 Parameter Candidates | R14-16 | 主交付 |
| R14-17 Release | R14-17 | 主交付 |

## 记录位置

阶段索引沿用 `docs/modular-rewrite/R14-prediction-review-analytics-ui/README.md`；记录以主交付 ID 命名，逐项列出原节点覆盖。索引在实际启动阶段时创建，合并项标为“并入主交付/待验”，不能单独伪造 DONE。

## 阶段出口与交接

- 17页中分配给本阶段的页面/状态及原业务ID分别有关闭记录；布局补差与完整挂载验证随本域完成，不能默认推给R16/R17。仅既定真实外部环境项可登记到R17-06，设计或行为失败不属此豁免。
- 各主交付及其合并验收项均有记录，真实功能/状态/写入/错误和恢复路径接通；无空壳或伪造数据。
- 只退出已替代的职责，当前调用方、依赖、测试和清单已同步；不为满足文件名清单删除仍正确的共享实现。
- 所有本阶段适用门禁通过。最终新库待验按原安排逐项登记 owner、条件和 R17-06 接收位置；不得标记完整业务验收通过。
- 阶段启动时在既有记录体系建立阶段索引；每批记录覆盖原 ID，阶段结束创建阶段完成记录并附实际 A/M/R/D、契约、验证和回退摘要。记录路径沿用原阶段目录名，历史记录不重写。
- R15 仅在本阶段出口满足及用户授权后进入 READY。

## 已有设计输入的追溯

以下映射已按10/31审计订正，独立设计记录存在不等于生产已完成同状态验收。以[接入处置表](../modular-rewrite/ui-integration-disposition-20261031.md)确定复用、局部补差和待验责任；采用最新已批准设计，不要求保留已证实不匹配的旧布局。新增设计决定仍须Figma先行，已有真实能力必须保留。

适用[总纲第 8.0 节](00-总体架构与前23节.md#80-figma-设计接入依据2026-10-08-补充)。以下设计记录来自 `ui-design-system` 分支 `docs/`，按最新明确 accepted 版本及实际节点读取，并作为对应 Atomic Task 的共同输入和验收依据。

| R14 节点 | Figma 页面与独立记录 | 实现边界 |
|---|---|---|
| R14-01～06 | S03 `prediction` / `FIGMA_SCREEN_PREDICTION.md`、`FIGMA_SCREEN_PREDICTION_STATE.json` | 上下文、readiness、正式/影子、结果及 P4 研究；输入、Provider、准入与输出分别维护，UI 不计算模型结果 |
| R14-07 | S05 `runs` / `FIGMA_SCREEN_RUNS.md`、`FIGMA_SCREEN_RUNS_STATE.json` | 列表、详情、技术追踪与历史隐藏；未知值不补零，隐藏不删除底层血缘 |
| R14-08～11 | S04 `review` / `FIGMA_SCREEN_REVIEW.md`、`FIGMA_SCREEN_REVIEW_STATE.json` | 九步工作流、事实、回包与结果；查看不授予执行许可，确认后的身份/SHA 仍须验证 |
| R14-12 | S04 `review` 与 S12 `analytics` 的赛后/结算相关状态 | 按 Postmatch 的实际职责核对覆盖，复盘与结算分别处理；缺少独立场景时在本节点补齐设计输入，不推测已有整页完成 |
| R14-13～16 | S12 `analytics` / `FIGMA_SCREEN_ANALYTICS.md`、`FIGMA_SCREEN_ANALYTICS_STATE.json` | 总览、任务、包/建议与候选；全局快照不授予精确分区许可，接受建议不等于正式能力/参数已修改 |
| R14-17 | S11 `release` / `FIGMA_SCREEN_RELEASE.md`、`FIGMA_SCREEN_RELEASE_STATE.json` | 验收请求、分类检查、不可变报告和历史；报告完成不等于真实发布，保留公开运行时警告 |

上述页面在 `460eb0d` 检查点已通过有限设计验收，不表示 R14 代码或真实后端联调完成。进入节点时读取其独立记录的未实现项和源码差异，只解决本节点真实影响范围；设计包含的手动演示结果必须替换为真实任务状态，未分发模型继续显示原有不可用语义，不伪造成功结果。

按实际操作验证上下文切换、输入失效、请求身份、取消/迟到结果、重试/结果未知、最终确认和只读边界；跨页导航保留正确来源与主题，不转移执行权限。Windows 视觉/交互证据沿用原体系，未实跑的私有引擎、PG 与 Full 验收继续登记待验；设计 PASS 不能冲销这些条件。
