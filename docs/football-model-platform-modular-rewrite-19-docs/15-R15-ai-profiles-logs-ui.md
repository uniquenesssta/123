# R15：AI 会话、凭据与诊断可靠性——执行任务书

> 2026-10-30 订正；代码审计基线：`2f3afc639b3961bf2859d9f4997ae66c9177ded0`。
> 与[总纲](00-总体架构与前23节.md)共同适用；[审计与全量映射](../modular-rewrite/task-plan-review-20261030.md)。
> 原任务 ID 保留追溯，本阶段改为 3 个可验收交付批次。任务合并不代表功能取消、实现完成或验收通过。

## 目标、当前依据与范围

复用 platform/tauri、timing、storage、diagnostics、app/lifecycle。请求取消继续调用现有后端请求注册表；本批不新增协议、工具调用、自动数据库写入或研究引擎。

当前来源：src/main.ts 中 AI 会话、配置与日志动作，src/pages/{apiWorkspace,openai,logs}.ts，相关 client/types 与现有 diagnostics。建议归属：`src/features/{ai-workspace,openai-profiles,logs}/`。这是职责定位，不是必须生成的目录清单；实施前读取当前代码，报告与基线的差异。

本阶段聚焦业务归属、真实流程与可靠性；本轮规划排除 UI 视觉改版、功能删减、模型改写和不相关后端重构。保留搜索/分页/焦点/滚动/草稿/主题/工作区恢复等现有用户行为。

## 执行卡使用前提

本阶段逐批执行下方详细卡，并同时读取[执行约定与验证命令](../modular-rewrite/remaining-execution-contract.md)。源码锚点来自 `0b1132bd265adbd7d689c5345986c412fffb3ba4`；执行时先核对差异，迁移后的准确路径以已完成前批记录为准。计划中的职责名是目标，不表示文件已存在。

## 状态与依赖

当前全部为 **BLOCKED / 计划未实施**。进入条件为 R14 已按适用出口完成及用户授权；不因本次改写自动 READY。下表顺序是默认交付顺序，必要时在已授权阶段内按真实依赖拆出更小批次并说明原因，不以任务数量或文件数为目标。

| 主交付 ID | 交付批次 | 前置 |
|---|---|---|
| R15-01 | 会话、草稿与请求全生命周期 | R14 阶段出口与 R15 启动授权 |
| R15-07 | 配置、示例解析、凭据与连接测试 | R15-01 |
| R15-12 | 问题日志与诊断关联 | R15-07 |

## 执行与验收规则

1. 先检查实际调用图和现有实现。每个批次列明“复用 / 迁移 / 修复 / 补验”；不得因为任务名称包含重构而重写正确算法或已交付基础能力。
2. 状态、工作流、API、传输 DTO 与展示模型分别明确 owner；一个模块内紧密相关的小函数可以共置，复杂独立职责再拆目录。不要求每域建立固定十层目录、一命令一文件、镜像空 CSS/测试目录。
3. 每批同时迁移本域 main 状态/动作、API/类型消费者和现有测试，切换唯一调用入口后退出已替代实现。跨域 DTO 选唯一契约 owner，跨域调用走公开接口；不复制类型或引入无期限兼容壳。共享 Foundation 保持唯一实现。
4. 保护现有功能、命令参数/返回/错误、Schema、历史迁移、存储键/版本、Excel/包格式、模型公式/参数/路由/时点和身份血缘。已证实缺陷单列修复并验证，不借兼容保留缺陷。公开不可用模型不能用假数据补成功。
5. 验证跟随每批实现：先执行受影响的行为/竞态/契约检查，再按影响完成现有前端、Rust、Windows 构建/运行门禁。跨层改动包含相应后端检查；不改 Rust 的批次可引用同一后端树已验结果并说明边界。失败必须修复，不能跳过/放宽检查。
6. 静态检查、类型/构建、自动测试、Windows 实际运行、真实外部数据分别记录。可复用证据必须标出提交或受测树摘要、环境、工具、入口和结果；只有代码/依赖/配置/资产均无相关变化才可复用。CI 只按精确提交一次性核对，不轮询。
7. 真实 PG/XLSX/Full 等沿用总纲已批准的最终新库安排。节点可记录“实现及 Automated 完成、真实环境待验”，但不能把它写成 Full PASS；阶段索引显式携带待验项到 R17-06，正式发布必须清零。其余本阶段必需门禁失败不得以延期绕过。
8. 进程 TEMP/TMP 使用项目临时目录并单独提醒配置；继续 Windows/PowerShell。不得新增持续回归框架、runner、workflow、专项数据库、生产依赖或公开私有资产。
9. 分支沿用每阶段一个既定分支；进入阶段同时需要前置出口和用户授权。本次任务书订正不自动授权 R13 开发。后续提交推送沿用用户长期授权，不逐批重复询问。
10. 每批记录实际变更清单、唯一 owner/调用链、验证与未验项、剩余风险和回退点。合并项可共用一份交付记录，但必须逐项列出覆盖证据；不能整组一勾代替业务验收。文档按有效信息组织，不强制填 23 节空模板。

## R15-01 会话、草稿与请求全生命周期

覆盖原节点：R15-01、R15-02、R15-03、R15-04、R15-05、R15-06。

**必要变更与边界：** 一起处理目录/搜索/选择/归档、历史消息、输入草稿、发送/等待/失败/重试/取消和只读上下文。草稿按会话身份归属，请求按 request ID 归属；保留已确认的发送锁策略，不擅自增加并发能力。

**业务验收：** A 请求期间浏览 B：不展示 A pending、不丢 B 草稿、不强切回 A；快速 A→B→C 迟到读取不覆盖 C；每次 await 后、写状态前重新检查身份；取消不等于终态、超时未知先核验；新会话不能复用旧请求确认；上下文主动勾选，聊天纯文本无工具/schema 写入。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/apiWorkspace.ts`、`src/main.ts`、`src/platform/tauri/index.ts`。代表性现存函数：`loadApiWorkspace`（main.ts:1976）、`sendApiWorkspaceMessage`（main.ts:2054）、`openTeamApiWorkspace`（main.ts:3235）、`openPlayerApiWorkspace`（main.ts:3254）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`listOpenAiProfiles`、`listApiWorkspacePresets`、`listApiWorkspaceSessions`、`playerCatalogReferenceData`、`readApiWorkspaceSession`、`sendApiWorkspaceMessage`、`readTeam`、`readPlayer`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

ai-workspace 的session选择、每会话draft、每request pending分别具名；发送锁按既有产品语义，request→originSession绑定直到终态。

目录选择→身份受控读取→草稿输入→显式发送→pending→响应/失败/取消→原会话结果。每次await后写状态前重新检查身份和生命周期。

### 原子实施顺序

1. 子步骤A：迁目录/详情读取与草稿存储，补快速选择和读取失败测试；不能假设现有全局draft已按会话隔离。
2. 子步骤B：迁发送/pending/错误恢复，保持已有发送锁；记录发送瞬间的来源、内容、profile和上下文。
3. 子步骤C：迁取消与迟到响应处理，区分本地过期丢弃和后端真实取消，不用destroy宣称后端已停止。
4. 子步骤D：接只读上下文、归档和跨页导航，验证B草稿不被A完成/取消清空；退出全局AI状态及事件分支。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R15-01-V1**：A发送后进入B并输入草稿，A成功/失败/取消→B草稿和选择保持，A消息归A且不强制跳回。
- **R15-01-V2**：A/B/C详情乱序返回→仅当前选择能写详情；当前读取失败不显示另一会话。
- **R15-01-V3**：A返回后等待列表刷新时切B→刷新完成仍重新判身份，不能用await之前缓存判断覆盖B。
- **R15-01-V4**：取消一个request→其他会话不受影响；超时未知先读回，不自动重发；聊天无工具/schema和隐式数据库写操作。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-api-workspace.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-01: verify-api-workspace failed' }
node scripts/verify-api-workspace-research-control.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-01: verify-api-workspace-research-control failed' }
node scripts/verify-ai-workspace-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-01: verify-ai-workspace-service failed' }
node scripts/verify-api-compatible-transport.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-01: verify-api-compatible-transport failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R15-07 配置、示例解析、凭据与连接测试

覆盖原节点：R15-07、R15-08、R15-09、R15-10、R15-11。

**必要变更与边界：** 分别归属配置目录/草稿/显式保存/启用、协议端点模型超时、API 示例解析/预览/确认、凭据写入替换清除与状态、独立连接测试。

**业务验收：** 解析不自动覆盖正式配置；配置保存/密钥存在/测试成功/真实请求成功分开判断；历史会话不静默换绑；敏感输入仅在必要的临时输入控件与凭据调用中存在，不进入可回显 ViewModel、截图、日志、持久化草稿；失败分类与取消准确。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/openai.ts`、`src/main.ts`、`src/app/lifecycle/asyncScope.ts`。代表性现存函数：`parseOpenAiApiExampleNow`（main.ts:6650）、`applyOpenAiApiExample`（main.ts:6623）、`saveOpenAiProfile`（main.ts:6716）、`testOpenAiProfile`（main.ts:6737）、`activateOpenAiProfile`（main.ts:6750）、`requestClearOpenAiKey`（main.ts:6785）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`parseOpenAiApiExample`、`saveOpenAiProfile`、`testOpenAiProfile`、`setActiveOpenAiProfile`、`clearOpenAiProfileKey`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

openai-profiles 目录/选择、编辑草稿、示例解析预览、凭据动作和测试请求分开；凭据由原Windows/后端存储管理。

示例文本→解析候选→显式应用草稿→显式保存；凭据动作独立；连接测试绑定profile及测试时版本，不能充当保存或启用。

### 原子实施顺序

1. 核对当前解析与应用触发，保留协议/端点/model/超时语义，确认显式应用边界是否已落实。
2. 迁配置草稿/目录和保存启用，再迁解析请求身份；不得解析成功就静默覆盖正式配置。
3. 迁凭据写入/替换/清除与独立连接测试，清除确认绑定当前profile，敏感输入及时释放。
4. 联验更换配置、关闭编辑和迟到解析/测试结果，保存失败保留安全草稿并退出main对应函数。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R15-07-V1**：输入示例A后改为B或关闭编辑→A迟到解析不能回填；应用候选仍需明确动作。
- **R15-07-V2**：保存失败→不能显示已启用；有密钥但连接测试失败→状态分别表达。
- **R15-07-V3**：确认清除A凭据时切到B→不允许误清B；截图、草稿恢复、日志和错误正文无密钥。
- **R15-07-V4**：历史会话引用缺失配置→明确不可用，不自动绑定另一个配置继续发送。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-openai-profile-ui.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-07: verify-openai-profile-ui failed' }
node scripts/verify-profile-editor-persistence.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-07: verify-profile-editor-persistence failed' }
node scripts/verify-api-compatible-transport.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-07: verify-api-compatible-transport failed' }
node scripts/verify-api-runtime-diagnostics.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-07: verify-api-runtime-diagnostics failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R15-12 问题日志与诊断关联

覆盖原节点：R15-12、R15-13。

**必要变更与边界：** 归属问题目录/详情/导出/清除/运行摘要，复用唯一 trace 与脱敏逻辑，核对前后端关联 ID；按实际风险补防刷屏验证。

**业务验收：** 重复错误受控且可追溯；导出脱敏，清除需确认；正文/凭据不泄漏；普通 UI 错误与请求错误不重复记账；真实 API/Windows 凭据/会话持久化/取消单列环境验收。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/logs.ts`、`src/platform/diagnostics/index.ts`、`src/platform/tauri/index.ts`。代表性现存函数：`loadIssueLogs`（main.ts:1943）、`recordClientIssue`（main.ts:891）、`userFacingError`（main.ts:866）、`technicalErrorMessage`（main.ts:881）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`listIssueLogs`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

logs feature持有筛选/详情/导出/清除请求；全局错误记录、trace、脱敏和去重仍归现有Platform及WarningCenter。

UI/transport错误→唯一记录入口→问题列表→详情/脱敏导出；清除只影响明确的日志集合，不清任务/草稿。

### 原子实施顺序

1. 区分通用错误上报与页面查询职责，盘点export/clear动作和原重日志抑制机制。
2. 迁日志目录、过滤、详情与动作，继续调用既有脱敏/关联ID工具，不复制一套summary。
3. 核对前后端trace关联与重复错误限流；导出前也做同一脱敏边界。
4. 验证清除确认、导出取消/写失败和跨页刷新，保留问题中心与任务状态独立。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R15-12-V1**：一次transport失败经过UI catch→可追溯但不重复刷屏；重复错误按既有策略合并。
- **R15-12-V2**：日志含敏感URL/凭据样例→列表、详情、导出都不泄漏；测试样例不得用真实密钥。
- **R15-12-V3**：取消清除→日志不变；确认清除→不影响后台任务、会话草稿或错误根因状态。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-api-runtime-diagnostics.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-12: verify-api-runtime-diagnostics failed' }
node scripts/verify-warning-center.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-12: verify-warning-center failed' }
node scripts/verify-api-workspace.mjs
if ($LASTEXITCODE -ne 0) { throw 'R15-12: verify-api-workspace failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## 原节点去向（完整覆盖）

| 原节点 | 当前主交付 | 处理方式 |
|---|---|---|
| R15-01 AI Session Directory | R15-01 | 主交付 |
| R15-02 Conversation | R15-01 | 合并验收项（非 DONE） |
| R15-03 Composer | R15-01 | 合并验收项（非 DONE） |
| R15-04 Pending Request | R15-01 | 合并验收项（非 DONE） |
| R15-05 Request Cancellation | R15-01 | 合并验收项（非 DONE） |
| R15-06 Context Picker | R15-01 | 合并验收项（非 DONE） |
| R15-07 OpenAI Profile Directory | R15-07 | 主交付 |
| R15-08 OpenAI Profile Editor | R15-07 | 合并验收项（非 DONE） |
| R15-09 API Example Parser | R15-07 | 合并验收项（非 DONE） |
| R15-10 Credential Actions | R15-07 | 合并验收项（非 DONE） |
| R15-11 Profile Test | R15-07 | 合并验收项（非 DONE） |
| R15-12 Logs | R15-12 | 主交付 |
| R15-13 Diagnostics Bridge | R15-12 | 合并验收项（非 DONE） |

## 记录位置

阶段索引沿用 `docs/modular-rewrite/R15-ai-profiles-logs-ui/README.md`；记录以主交付 ID 命名，逐项列出原节点覆盖。索引在实际启动阶段时创建，合并项标为“并入主交付/待验”，不能单独伪造 DONE。

## 阶段出口与交接

- 各主交付及其合并验收项均有记录，真实功能/状态/写入/错误和恢复路径接通；无空壳或伪造数据。
- 只退出已替代的职责，当前调用方、依赖、测试和清单已同步；不为满足文件名清单删除仍正确的共享实现。
- 所有本阶段适用门禁通过。最终新库待验按原安排逐项登记 owner、条件和 R17-06 接收位置；不得标记完整业务验收通过。
- 阶段启动时在既有记录体系建立阶段索引；每批记录覆盖原 ID，阶段结束创建阶段完成记录并附实际 A/M/R/D、契约、验证和回退摘要。记录路径沿用原阶段目录名，历史记录不重写。
- R16 仅在本阶段出口满足及用户授权后进入 READY。

## 已有设计输入的追溯

以下保留原页面和能力映射供查阅；旧检查点的“尚未开始/有限验收”不是当前设计状态。此次排除 UI 改版，不重新制作已批准页面。业务重构保持现有视觉及交互合同；确需改变可见布局或交互方案时，先核对最新 Figma accepted 记录并取得适用设计，再实施该改变。

适用[总纲第 8.0 节](00-总体架构与前23节.md#80-figma-设计接入依据2026-10-08-补充)。以下文件均来自 `ui-design-system` 分支 `docs/`，执行前核对最新明确 accepted 检查点和实际节点，作为对应 Atomic Task 的设计输入及完成条件。

| R15 节点 | Figma 输入 | 本次核对状态 / 接入要求 |
|---|---|---|
| R15-01～06 | S13 `api_workspace`：`FIGMA_SCREEN_API_WORKSPACE.md`、`FIGMA_SCREEN_API_WORKSPACE_STATE.json`；通用结构见 `FIGMA_AI_CHAT.md` | 视觉、代表性状态与有限原型已验收；接入真实会话、消息、草稿、请求/取消和只读上下文 |
| R15-07～11 | S14 `openai`：从 `FIGMA_SCREENS_PLAN.md`恢复 | 设计尚未开始；配置目录/编辑、API 示例解析、凭据操作和配置测试需在相应节点验收前取得 accepted 设计与状态记录 |
| R15-12～13 | S16 `logs`：从 `FIGMA_SCREENS_PLAN.md`恢复；复用 Shell 与反馈组件 | 设计尚未开始；日志页面与诊断入口的可见部分需取得 accepted 设计，内部诊断桥按原接口职责实施 |

表内未开始状态来自 `460eb0d` 检查点，不能把旧空白/WIP 恢复文件当作 S14 或 S16 已完成；后续以总计划和对应实际交付为准。内部非视觉工作可以先核对，不能用临时界面替代最终视觉验收，也不要求在 R8 阶段提前完成这些页面。

S13 的预置 A/B/NEW 状态用于说明请求身份和草稿隔离，真实实现须验证：A 发送期间浏览 B 不显示 A 的待处理消息且不丢 B 草稿；A 完成不强制切离 B；读取失败不显示其他会话；取消请求不等于取消终态；结果未知先核验，不自动重发。沿用该页已确认发送锁语义、配置身份、主动勾选上下文、归档保留审计及历史只读边界，具体命令行为与 R9/R11 契约核对。

配置已保存、密钥存在、配置测试通过与某次真实请求成功分别表达；已有会话不能因配置缺失而静默换绑。密钥不进入普通 UI 状态、截图、日志或恢复记录。Windows 验证复用原入口，分别记录代表性 UI/模拟契约结果与真实请求/持久化结果；Figma 设计验收不代表网络、凭据或数据库已经验证。
