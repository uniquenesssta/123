# R13：基础数据与编排业务模块化——执行任务书

> 2026-10-31 接入审计后订正；代码基线：`6e1361210ad4b9fd9f6dd969fad112397e84f6ff`。
> 与[总纲](00-总体架构与前23节.md)共同适用；[审计与全量映射](../modular-rewrite/task-plan-review-20261030.md)。
> 原任务 ID 保留追溯，本阶段为 7 个业务交付批次及 1 个设计补差批次R13-UI01，共8批。任务合并不代表功能取消、实现完成或验收通过。

> 最新责任与证据边界见[17页/40项接入处置表](../modular-rewrite/ui-integration-disposition-20261031.md)。R12及已完成业务批次不重开；用户最新明确启动R13-07，本批已DONE，UI01已获授权执行中，状态以执行索引为准。

## 目标、当前依据与范围

复用 R12 的路由、Workspace、生命周期、Forms、确认弹窗、文件选择和样式入口。Competitions 与 Rules 共享现有页面入口，不新增导航项；跨域跳转传稳定 ID 与来源上下文。

当前已存在 `src/features/{dashboard,database,competitions,rules,teams,players,architecture}/`，直接复用其公开入口与实际owner；剩余业务入口为 `src/pages/{lineupPresets,lineups,workbooks}.ts`、`src/main.ts`、`src/api/client.ts` 及类型消费者。预设、阵容、工作簿仅按实际职责迁移，不预造空目录。

本阶段交付已批准设计的局部补差、必要业务归属/缺陷修复及缺失验证；不重新设计视觉方向，不从零重建已存在页面。保留搜索/分页/焦点/滚动/草稿/主题/工作区恢复和真实数据能力，不删功能、不改模型、不扩大无关后端重构。设计义务与业务义务分别验收，不能以保留旧HTML代替最新设计接入。

## 跨批次所有权

- 球队资料包、阵容导入、工作簿入口调用同一业务工作流；R13-05/07先迁移其真实owner，R13-09复用并完成其余格式与联验，不再次重写。通用文件选择留在Platform，业务预检/提交不放通用层。
- 公共传输DTO保持唯一契约定义，Feature自己的草稿与ViewModel本域所有；跨域来源/身份通过公开接口传递。基础连接与全局摘要复用Session，不再建第二套全局状态。

## 执行卡使用前提

本阶段逐批执行下方详细卡，并同时读取[执行约定与验证命令](../modular-rewrite/remaining-execution-contract.md)。下方旧函数行号来自历史 `0b1132bd265adbd7d689c5345986c412fffb3ba4`，只作追溯，不能直接当当前定位；执行时在当前源码按符号追踪实际调用方。最新接入审计基线及前批记录优先，已建立的Feature直接复用，未建立的目标职责按实际需要拆分。

## 状态与依赖

R13-01、02、03（含04）、05、06及原10随01的业务交付均DONE，证据见[阶段执行索引](../modular-rewrite/R13-core-ui-features/README.md)。R13-07已正式DONE（覆盖原07/08），精确CI/双bundle/startup见阶段索引；用户最新已授权R13-UI01，承接已交付6页设计补差，当前执行中，R13-09未启动。当前顺序UI01→09；UI01不是07的代码前置，不因调整次序重做已交付业务；R12保持结束。

| 主交付 ID | 交付批次 | 前置 |
|---|---|---|
| R13-01 | 只读入口与模块接入试点 | R12 已确认结束；前期基线核对已完成；R13-01已获授权并正式DONE |
| R13-02 | 数据库连接与危险操作 | R13-01 |
| R13-03 | 赛事层级、规则与路由 | R13-02 |
| R13-05 | 球队目录、档案与资料包 | R13-03 |
| R13-06 | 球员目录、履历与观察 | R13-05 |
| R13-UI01 | 已交付6页设计补差与同状态验收 | R13-06 DONE；最新批准设计；启动授权 |
| R13-07 | 预设与双方阵容闭环 | R13-06 DONE；用户最新已授权，UI01独立保留 |
| R13-09 | 工作簿导出、预检与提交 | R13-07 |

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

## R13-01 只读入口与模块接入试点

覆盖原节点：R13-01、R13-10。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-01、BA-16**。本项已按报告核对基线差异并获启动授权；实际结果见[实施记录](../modular-rewrite/R13-core-ui-features/R13-01-read-only-entry-pilot.md)。R12按用户确认保持结束，不以旧索引重新安排R12任务。

**必要变更与边界：** 完成首页总览、数据库健康、最近运行、注意事项、快捷动作及架构只读信息的归属。现有简单 View 可保留原逻辑迁入对应模块；从 manifest/现有 DTO 读取，不复制架构清单或创建不存在的指标。

**业务验收：** 冷启动、未配置/连接失败/有数据状态；快捷入口命中现有路由；架构信息不可写；验证试点能直接消费现有 Session，未创建第二个摘要状态。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/dashboard.ts`、`src/pages/architecture.ts`、`src/bootstrap/registerApplicationModules.ts`。代表性现存函数：`render`（main.ts:488）、`initializeApplication`（main.ts:8465）、`loadNavigationPage`（main.ts:2179）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

注意：本批锚点包含全局分派函数，以下 API 盘点也包含其他页面分支，仅用于识别共享边界，不属于本批迁移清单。本批只迁首页与架构对应分支，其他领域仍归各自后续批次。

上述函数直接调用的现有 API：`listIssueLogs`、`listOpenAiProfiles`、`listReleaseAcceptanceRuns`、`readReleaseAcceptanceRun`、`listCoaches`、`listFormations`、`readTeam`、`listTeamMatchLineups`、`listTeamLineupPresets`、`playerCatalogReferenceData`、`readP4MatchWorkspace`、`readP4TaskWorkspace`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

dashboard 与 architecture 的只读呈现模块；Session 继续拥有 Bootstrap 摘要，Router 继续拥有路由。

Session 快照→页面投影→快捷导航意图→既有 Router；本域不建立可写全局摘要，不把运行记录缓存放入首页 View。

### 原子实施顺序

1. 先逐条列出两个页面传入字段、快捷动作和 render 分支，冻结现有路由键及空值展示。
2. 为现有两个 View 建立具名公开入口；仅抽取其本域投影和事件转发，不搬迁全局 render。
3. 在现有 render/loadNavigationPage 中切换这两个入口，保留尚未迁移页面，记录同一功能唯一实现。
4. 复用 AppSession/Router/MountHandle 的接口，验证后把试点接入方式作为本阶段其他批次的示例。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R13-01-V1**：无数据库启动→显示未配置及可用设置入口，不能伪造统计；连接成功刷新后只消费新摘要。
- **R13-01-V2**：连续进入/离开首页两次并点一次快捷动作→只发生一次导航，监听器无重复；焦点和来源可恢复。
- **R13-01-V3**：读取架构信息→内容来自现有真实资料；界面不允许修改保护清单。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-browser-bootstrap.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-01: verify-browser-bootstrap failed' }
node scripts/verify-dual-level-navigation.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-01: verify-dual-level-navigation failed' }
node scripts/verify-core-workspace-hierarchy.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-01: verify-core-workspace-hierarchy failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R13-02 数据库连接与危险操作

当前实现及逐层验证见[02实施记录](../modular-rewrite/R13-core-ui-features/R13-02-database-connection-and-reset.md)。本地门禁通过不等于本批CI、安装包或真实新库重置已验收。

覆盖原节点：R13-02。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-01、BA-14**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 分离连接草稿、连接/健康查询、迁移状态与重置流程；连接结果只投影到现有 Session，重置先预检再明确确认。

**业务验收：** 凭据/URL 脱敏；连接切换迟到结果不覆盖新连接；失败保留草稿；取消确认无写入；重置确认身份与提交对象一致。真实重置只在授权的新测试库验收。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/database.ts`、`src/app/session/index.ts`、`src-tauri/src/commands/database/reset/command.rs`。代表性现存函数：`connectDatabase`（main.ts:1386）、`requestDatabaseReset`（main.ts:1398）、`executeDatabaseReset`（main.ts:1404）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`configureDatabase`、`resetDatabase`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

database 连接草稿和重置工作流；连接生效后的摘要由既有 Session 发布。密码只存在必要的临时输入/调用边界。

连接输入→现有 API→Session 刷新；重置意图→身份确认→单次提交→真实成功后重载。连接变更使旧查询和未提交重置确认失效。

### 原子实施顺序

1. 核对连接/重置参数、确认条件、日志脱敏、偏好保存及重置后的导航行为，记录当前接口签名。
2. 抽出连接草稿与连接动作，再抽出重置确认/执行；后端事务和重置实现保持原 owner。
3. 绑定对当前连接身份的校验和重复提交锁；所有成功提示来自真实响应，失败保留非敏感草稿。
4. 切换按钮/表单分派，撤出 main 对应函数与状态，验证记录不含连接密码或完整 URL。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R13-02-V1**：连接 A 慢、随后连接 B 快→A 迟到不能把摘要恢复到 A；各自失败显示正确对象。
- **R13-02-V2**：打开重置确认后切换连接→旧确认不能重置新库；关闭确认→后端重置调用次数为零。
- **R13-02-V3**：重置请求失败→不清空本地对象并宣称成功；提交结果未知→核查数据库状态后决定恢复，禁止自动再重置。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-database-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-02: verify-database-service failed' }
node scripts/verify-database-reset.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-02: verify-database-reset failed' }
node scripts/verify-api-runtime-diagnostics.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-02: verify-api-runtime-diagnostics failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R13-03 赛事层级、规则与路由

覆盖原节点：R13-03、R13-04。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-01、BA-14**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 分别归属赛事目录/详情/赛季/阶段/轮次、规则包目录/编辑/校验、绑定及路由预览；共享页面组合保持，规则合法性与正式模型路由由后端决定。

**业务验收：** 父级变化使不适用子级与预览失效；编辑后上下文保持；关联引用与删除保护；前端提示不能绕过后端规则校验；现有赛事类型和绑定语义保持。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/rules.ts`、`src/pages/rules/catalogueFilters.ts`、`src/controllers/pageLoaders.ts`。代表性现存函数：`createCompetition`（main.ts:1507）、`createSeason`（main.ts:1596）、`createStage`（main.ts:1613）、`createRound`（main.ts:1630）、`registerRulePackage`（main.ts:1647）、`createBinding`（main.ts:1656）、`previewRoute`（main.ts:1477）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`createCompetition`、`createSeason`、`createStage`、`createRound`、`registerRulePackage`、`createCompetitionBinding`、`previewRoute`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

competitions 拥有赛事层级选择/编辑草稿；rules 拥有规则包与绑定草稿/路由预览。页面 rules 只组合二者。

层级选择→稳定 ID→目录/详情查询；包草稿→后端验证/登记→绑定刷新。层级改变必须失效关联子级和旧路由预览。

### 原子实施顺序

1. 列出赛事/赛季/阶段/轮次和规则包/绑定的读取写入调用，标记共同页面的事件归属。
2. 先迁移赛事层级查询与编辑，再迁移规则包/绑定；共同只读类型归单一契约 owner。
3. 将目录筛选与实体写入分开；路由预览消费后端结果，不能在本域新增模型选择规则。
4. 接回同一 rules 路由和已批准布局，更新相关守卫的真实源码路径并退出旧实现。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R13-03-V1**：选择赛事 A 的阶段后换赛事 B→A 子级不再可提交；慢返回不覆盖 B。
- **R13-03-V2**：后端拒绝非法规则或引用中的删除→保留错误与草稿，列表不伪造删除成功。
- **R13-03-V3**：改绑定后再次预览→展示新绑定对应结果；原预览不能作为正式执行许可。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-competition-hierarchy.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-03: verify-competition-hierarchy failed' }
node scripts/verify-competition-bindings.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-03: verify-competition-bindings failed' }
node scripts/verify-route-resolution.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-03: verify-route-resolution failed' }
node scripts/verify-competition-rules-service.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-03: verify-competition-rules-service failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R13-05 球队目录、档案与资料包

覆盖原节点：R13-05。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-05、BA-07**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

实施进度：[05记录](../modular-rewrite/R13-core-ui-features/R13-05-team-catalog-profile-and-package.md)。本批四个状态owner和全部原入口已迁入实际Teams Feature，本地96步前端、最终42项生产行为、Windows浏览器/原生读取及类型构建通过；自身CI/安装包尚待核验，当前VERIFYING，06不提前启动。

**必要变更与边界：** 收拢球队目录/身份/名称/档案/名单/教练任期/阵型使用/阵容历史/编辑/删除/资料包状态和动作。读取、编辑草稿、删除预检及包预览分别拥有状态；复用现有公共控件。

**业务验收：** 搜索分页选择互不串用；中文名/别名与有效任期正确；切换对象的迟到读取丢弃；删除引用保护；preview/commit 身份与内容一致，失败可恢复；转球员只传稳定身份。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/teams.ts`、`src/controllers/pageLoaders.ts`、`src/types.ts`。代表性现存函数：`loadTeamCatalog`（main.ts:1865）、`openTeam`（main.ts:2484）、`updateTeam`（main.ts:2714）、`saveTeamProfile`（main.ts:2760）、`previewTeamPackageImport`（main.ts:2557）、`commitTeamPackageImport`（main.ts:2634）、`requestForceDeleteTeam`（main.ts:3037）、`confirmForceDeleteTeam`（main.ts:3067）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`listCoaches`、`listFormations`、`listTeamMatchLineups`、`listTeamLineupPresets`、`readTeam`、`updateTeam`、`addTeamName`、`playerCatalogReferenceData`、`upsertTeamProfile`、`chooseExcelImportFile`、`previewTeamPackageImport`、`commitTeamPackageImport`、`previewForceDeleteTeam`、`forceDeleteTeam`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

teams 下目录查询/分页/选择、详情与档案草稿、教练/阵型使用、删除预检和资料包分别具名；Workspace 保持标签与布局 owner。

目录查询→列表选择→详情；编辑/删除/包提交绑定 team ID 和预检版本。教练任期、名称、名单和历史分别投影，不从空值推算伪数据。

### 原子实施顺序

1. 盘点 main 中球队所有状态、事件分支、函数调用及 API；同时列出名称、教练、阵型、历史和普通/强制删除，不只迁代表函数。
2. 先迁目录/详情读取及失效保护，再迁编辑和档案子流程；共享足球文本映射按实际消费者保留唯一实现。
3. 迁移删除预检/最终确认和资料包 preview/resolve/commit，参数/身份/SHA 与后端契约一致。
4. 接通球队→球员来源返回、标签页恢复与刷新；R13-09只能复用本批包工作流，不能复制。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R13-05-V1**：A详情延迟、选择B→显示B或B错误，不能标成B却展示A；搜索后页码/游标按现有契约重置。
- **R13-05-V2**：包A预览后选择包B或球队B→旧确认失效；提交失败保留可恢复的预览和冲突选择。
- **R13-05-V3**：普通删除遇引用→明确拒绝；强制删除确认对象改变→阻止提交；取消确认无写入。
- **R13-05-V4**：球队跳球员再返回→保留正确球队ID/来源和工作区；缺失能力、名称或履历不补造。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-team-directory-detail.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-05: verify-team-directory-detail failed' }
node scripts/verify-team-names-profiles.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-05: verify-team-names-profiles failed' }
node scripts/verify-force-team-delete.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-05: verify-force-team-delete failed' }
node scripts/verify-team-package-preview-recovery.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-05: verify-team-package-preview-recovery failed' }
node scripts/verify-team-package-real-import-recovery.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-05: verify-team-package-real-import-recovery failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R13-06 球员目录、履历与观察

覆盖原节点：R13-06。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-05、BA-07**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

进度：[06实施记录](../modular-rewrite/R13-core-ui-features/R13-06-player-catalog-history-and-observations.md)。四个状态owner、完整现存调用迁移与缺陷修复已接通，30项行为/26浏览器/原生读取通过；完整96步前端/类型/构建通过，自身精确CI/包尚待验。

**必要变更与边界：** 分离球员身份/名称/位置、球队履历、伤停、能力观察、动态标签、比赛历史与编辑删除；共享字段类型只有一个定义。比赛历史按实际接口核对，既有阵容来源/返回与历史投影全部保留，不虚构PlayerDetail没有的独立比赛历史API。

**业务验收：** 直接进入目录清除不适用的球队来源；位置映射正确；效力区间/伤停/能力草稿独立；标签不覆盖长期能力；批次失败/部分失败明确呈现，禁止误报全部成功。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/players.ts`、`src/controllers/pageLoaders.ts`、`src/components/footballText.ts`。代表性现存函数：`loadPlayerCatalog`（main.ts:1759）、`searchPlayers`（main.ts:2307）、`openPlayer`（main.ts:2393）、`updatePlayer`（main.ts:3473）、`addPlayerTeamPeriod`（main.ts:3621）、`addPlayerAvailability`（main.ts:3644）、`addPlayerAbility`（main.ts:3672）、`addPlayerDynamicTag`（main.ts:3719）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`readPlayer`、`updatePlayer`、`addPlayerName`、`listPlayers`、`addPlayerTeamPeriod`、`addPlayerAvailability`、`addPlayerAbilityObservation`、`addPlayerDynamicTag`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

players 的目录/游标/选择、详情、身份编辑、位置、履历、伤停、能力观察、动态标签各有明确 owner；来源导航使用现有 NavigationContext。

查询条件→游标列表→player ID详情；各类编辑草稿分别提交到既有命令，刷新只影响相应对象。标签与长期能力不是同一状态。

### 原子实施顺序

1. 枚举球员相关状态、表单和动作，包含外部ID、位置、名称、批量操作与贡献查询，冻结字段/空值语义。
2. 迁移查询、分页和来源上下文，再迁移详情/编辑工作流；对重复足球映射选择已有唯一 owner。
3. 分别接通履历/伤停/能力/标签写入，保留原时间区间、身份与后端校验。
4. 更新调用方和测试路径，验证离开/重返页面草稿及来源恢复后退出 main 对应实现。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R13-06-V1**：从球队A进入球员后直接打开球员目录→不携带不适用的球队限制；前进后退游标对应当前查询。
- **R13-06-V2**：修改伤停后能力观察不变；提交标签不覆盖能力；失败只标记对应草稿，不清空所有编辑。
- **R13-06-V3**：批量操作部分成功→按真实项目结果呈现，不把失败项移除并报告全部成功。
- **R13-06-V4**：连续选A/B，A迟到→不能覆盖B身份、图片/名称、履历或表单。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-player-pagination-team-filter.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-06: verify-player-pagination-team-filter failed' }
node scripts/verify-player-navigation-context.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-06: verify-player-navigation-context failed' }
node scripts/verify-player-role-inheritance.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-06: verify-player-role-inheritance failed' }
node scripts/verify-player-team-periods-availability.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-06: verify-player-team-periods-availability failed' }
node scripts/verify-player-abilities-dynamic-tags.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-06: verify-player-abilities-dynamic-tags failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R13-UI01 已交付页面的设计补差与同状态验收

**状态：IN_PROGRESS / 用户已启动。** 07已正式收尾；前置为R13-06已收尾、最新已批准设计及用户启动指令。承接原01/02/03/04/05/06/10的剩余视觉/内容义务，不改变这些业务批次DONE，也不是新的第68项产品功能。范围仅S01首页、S06球队、S07球员、S10赛事规则、S15数据库、S17架构。

### 直接复用、必要改变与不做项

复用当前Feature的生产数据/动作、R12 Shell/控件/token、05/06导入与草稿owner、02连接/重置防护。已有行为测试和交付结果保留；不重新迁这些业务，不重建页面/API，不操作真实重置/强删来验证外观。

| 子范围 | 已有基础 | 必要处理/关闭标准 |
|---|---|---|
| S01 | 渐变快捷卡、2+3布局、四色统计 | 同数据核对卡高、文字/链接层级和统计；批准差异才改，不硬编码真实数值 |
| S06/S07 | 固定筛选、目录、档案/速览、分页 | 目录与档案/速览按同对象核验卡片动作、完整度/名单、筛选层级和焦点；保留已接通业务 |
| S10 | 固定体系/地区与两列赛事网格 | 工具栏/路由说明同状态核验；仅具体赛事滚动，长名与更多结果不推走分类栏 |
| S15 | 02真实连接/危险操作及身份防护 | 运行概览身份/四指标/边界提示、连接/统计/危险分区按批准设计接入；必要数据无来源显示未取得，不虚构迁移数 |
| S17 | 01只读四层职责/历史/路由 | 补“架构声明不等于运行能力”、历史与运行时边界；内容取真实契约，不把私有引擎写成公开已具备 |

### 实施顺序与证据

1. 读取处置表和对应独立设计记录，列当前生产符号/Frame/状态；完整挂载同数据对照，先排除控件增强或恢复步骤缺失导致的假差异。只确认差异，不一次性改所有CSS。
2. 优先复用已有tokens/组件修共同根因；页面组合分别归对应Feature，禁止为视觉修复撤回业务owner。按六个子范围留可审阅diff和回退点；共享改动先检查受影响页面。
3. 对照[共享设计合同](../modular-rewrite/remaining-execution-contract.md#最新设计接入与同状态验收合同)执行同状态、明暗、尺寸、长内容、滚动和实际操作验证。原HTML等价测试因批准布局变化而调整时，必须保留原业务断言并附设计依据。
4. 每页分别给关闭证据；没有差异的子项用结果核销，不制造代码改动。全部六页适用状态通过后UI01才DONE；不顺带开始其他未授权任务；07已按用户后续指令独立恢复。

### 可操作验收场景

- **R13-UI01-V1**：1440×900同数据/主题打开六页→字号字重、卡片、工具栏和主动作与批准Frame对应；不得以“字体已加载”代替实际层级检查。
- **R13-UI01-V2**：960/1280/1920宽度、长球队/球员/赛事名、更多列表结果→栏位固定、列线对齐、间距遵循token；仅指定区域滚动，分页与关闭动作可达。
- **R13-UI01-V3**：球队/球员目录→速览→完整档案→返回，筛选/分页/对象/焦点保持；调用原Feature，不创建第二份对象状态。
- **R13-UI01-V4**：数据库未配置/已连接/失败、统计未知与危险分区→不补零、不复用过期身份，不因改布局绕过02确认保护；危险写入仅原授权环境。
- **R13-UI01-V5**：架构与数据库能力说明→与当前公开Provider和历史规则一致；没有源数据的指标不编造，原型专用验收按钮不搬进产品。

### 现有入口与停止条件

沿用 `scripts/design-refresh/productionFixture.mjs`、`scripts/design-refresh/browserFixture.ts` 和 `scripts/design-refresh/verifyApprovedDesign.mjs`；现有脚本只覆盖5路由的8种场景，须按实际受影响状态补充，不能把它当已覆盖数据库/架构。完整挂载与Windows原生命令验证仍另行记录；不新建第二套runner。完成受影响行为/视觉后按共享门禁验证前端构建及实际运行，真实外部项保持BA账本。

设计来源不明确、必要能力失联、未核实写入身份、必需状态未验或共享组件回归均阻塞本批。记录写入 `docs/modular-rewrite/R13-core-ui-features/R13-UI01-approved-screen-deltas.md`（实施时创建）；本次规划不预造通过记录。

## R13-07 预设与双方阵容闭环

### 审计后的实际交付范围

- **直接复用并核销重复建设：** 比赛目录/编辑、双边阵容、预设编辑/复制/归档/套用、历史及链路API均已存在；公共控件、双边基础布局和后端paired事务沿用。
- **本批必须完成：** S02 Matches紧凑详情、S08未选择/选定/编辑/套用状态按批准Frame补差；先核对完整挂载，再修需变动的布局。固定提交前的比赛、窗口、capturedAt及双方draft快照，await后核对身份；处理BA-03保护拒绝和必要owner归属。
- **增量验收：** R13-07-D1：链路读取A延迟时切B或改窗口→不得用A校验提交B草稿；同队跨比赛旧名单不回填。D2：预览后换对象/输入→确认失效且不发写入。D3：同状态检查比赛详情、长角色缩写、两侧行列、内部滚动及预设空态；所有动作仍调用原真实命令。

覆盖原节点：R13-07、R13-08。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-02、BA-03、BA-04、BA-06**。特别检查BA-03：含model run引用的比赛删除与0041保护冲突，在本批交付前明确保护拒绝及错误语义，补相应生产路径回归；不能只迁移原SQL，也不能放宽历史触发器。真实PG部分仍按报告闭合。

**必要变更与边界：** 分别拥有预设目录/编辑/成员/复制/归档/预览，以及比赛目录/编辑、双方编排、选人/角色/阵型、链路校验、历史和导入。双边提交协调器拥有一次提交状态；两侧查询分别有请求序号。

**业务验收：** 预设套用先预览，不改原预设；切比赛/球队丢弃旧名单；原子提交两侧，失败无半条阵容；保存不等于模型准入；角色/首发/历史行为兼容；导入失败保留当前草稿。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/lineups.ts`、`src/pages/lineupPresets.ts`、`src/types.ts`。代表性现存函数：`loadPairedLineupSide`（main.ts:4332）、`createPairedLineups`（main.ts:5197）、`previewApplyLineupPreset`（main.ts:4996）、`applyLineupPreset`（main.ts:5019）、`saveTeamLineupPreset`（main.ts:4912）、`createMatch`（main.ts:5431）、`requestRemoveLineupHistory`（main.ts:5469）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`listPlayers`、`listTeamLineupPresets`、`readMatchLineupChain`、`createLineupPair`、`listLineups`、`previewTeamLineupPresetApplication`、`saveTeamLineupPreset`、`createMatch`、`playerCatalogReferenceData`、`removeLineupHistory`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

lineup-presets 拥有预设定义/编辑/复制/归档；lineups 协调器拥有比赛和双边待提交副本，两侧查询独立序号；UI角色编辑不能直写预设。

比赛/球队→两侧名单→草稿校验→预设预览/应用副本→paired API单次原子提交→读取链路。保存成功与模型准入分开。

### 原子实施顺序

1. 分别冻结预设和阵容命令、角色/阵型映射与历史动作，列出两侧状态和影响名单的输入。
2. 先迁预设编辑/预览与读取，再迁比赛选择、双边名单查询和草稿；按side+match/team绑定响应。
3. 迁移副本应用与原子保存，保留单边旧合法入口；禁止把双边API改成两个独立写入。
4. 联验新增比赛、导入、设置球员、复制/归档预设、历史与回到推演入口；清理旧分派。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R13-07-V1**：主侧A慢请求后换比赛B→A不能填进B，客侧仍独立；切侧不丢另一侧草稿。
- **R13-07-V2**：应用预设后编辑本场球员→原预设不变；预览后换比赛→必须重算预览。
- **R13-07-V3**：双边保存后端失败→无一侧成功的虚假UI；实际PG验证无半条阵容，模拟原子响应不代替PG证据。
- **R13-07-V4**：首发数量/重复人/角色不合法→阻止错误提交并定位；保存后仍需单独检查推演准入。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-match-lineup-chain.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-07: verify-match-lineup-chain failed' }
node scripts/verify-lineup-preset-editor.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-07: verify-lineup-preset-editor failed' }
node scripts/verify-stage-e2-lineup-presets.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-07: verify-stage-e2-lineup-presets failed' }
node scripts/verify-lineup-scroll-continuity.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-07: verify-lineup-scroll-continuity failed' }
node scripts/verify-match-workflow-ui.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-07: verify-match-workflow-ui failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## R13-09 工作簿导出、预检与提交

### 审计后的实际交付范围

- **直接复用并核销重复建设：** 三类工作簿页面、Team/Player已在05/06交付的导入owner、文件选择和真实导出/预检/提交API；不把已完成流程搬回第二套共享状态。
- **本批必须完成：** S09分类竖排、导出/预检/确认面板和底部确认区接入；完成Match及共享入口的剩余编排。每个入口明确目标/文件摘要/票据、busy、重复和结果未知处理。
- **增量验收：** R13-09-D1：Team→Player→Match切换时旧预检不得驱动新提交；失败保留可恢复草稿。D2：同数据Idle/预检成功/冲突/busy/结果未知状态验证底部动作、焦点和独立滚动；真实XLSX→PG读回沿BA-07，不用JSON fixture核销。

覆盖原节点：R13-09。

### R1～R12遗留项承接

读取[前期遗留项基线核对](../modular-rewrite/prior-stage-baseline-20261030.md)：**BA-02、BA-04、BA-07**。按基线报告区分本批必须处理的风险与已批准的真实环境待验；前者未解决不得交付，后者逐项保留承接和证据，不能只标记“已检查”。

**必要变更与边界：** 按球员目录、球队资料包、球队/月度球员、比赛阵容、复盘包划分业务 owner；统一文件能力仍在 Platform，各业务 workflow 分离预览、冲突和提交状态。复盘事实写入继续交给 R14 对应 owner，不另建相同实现。

**业务验收：** 有效 XLSX 往返、外部 ID、中文名、重复行、冲突、失败恢复和批次审计；预览 A 后替换为 B 不可沿用 A 的确认；格式/Schema/错误语义保持。涉及复盘工作簿的跨阶段调用在 R14-08 联验。

**完成依据：** 本节业务断言逐条有生产调用路径与验证证据；原有功能均有保留映射；涉及持久化的真实未验项按执行规则登记，不能以模拟结果抵销。失败修复后复验受影响范围，提交前核对实际 diff 与回退点。


### 源码锚点与调用盘点

`src/pages/workbooks.ts`、`src/api/client.ts`、`scripts/verify-monthly-workbooks.mjs`。代表性现存函数：`previewPlayerImport`（main.ts:3529）、`commitPlayerImport`（main.ts:3564）、`previewTeamImport`（main.ts:2665）、`commitTeamImport`（main.ts:2700）、`previewMatchImport`（main.ts:4131）、`commitMatchImport`（main.ts:4168）、`exportAiMatchPackage`（main.ts:3798）。这些是入口锚点，不是穷尽迁移清单；还需盘点对应事件分支、辅助函数与全部调用方。

上述函数直接调用的现有 API：`chooseExcelImportFile`、`previewPlayerCatalogImport`、`commitPlayerCatalogImport`、`readPlayerCatalogImportPreview`、`previewTeamMonthlyImport`、`commitTeamMonthlyImport`、`readTeamMonthlyImportPreview`、`chooseZipImportFile`、`previewAiMatchPackage`、`previewMatchLineupImport`、`commitMatchLineupImport`、`readMatchLineupImportPreview`、`listLineups`、`chooseZipExportFile`、`exportAiMatchPackage`；间接调用须沿当前调用图补齐。参数、返回、错误码及 wire 命令从实际 client 和 command-contract 对照，不按名称猜测。

### 目标归属与数据流

workbooks 只组合各域公开导出/预检/提交能力；teams/players/lineups 各自持有批次和冲突状态，通用文件选择在 Platform。

选择文件→真实格式预检→批次/行身份与冲突→明确确认→commit→真实汇总/审计。包类型、源文件和实体身份共同约束确认。

### 原子实施顺序

1. 建立格式×导出/预览/冲突/提交/恢复矩阵，列明球队/球员目录、资料包、月度、比赛阵容、AI和复盘包的实际入口。
2. 直接复用前批已有包工作流，迁移剩余导出/预检动作；不复制领域状态到workbooks页面。
3. 切换入口、取消文件选择、替换文件时清晰失效旧预检；保持现有格式版本、行号与外部ID。
4. 用真实有效样例联验已具备链路，复盘提交部分登记到R14-08；存在环境缺口按真实待验账本记录。

每一步先确认入口可用与受影响断言，失败停在本步修复。批次内可以保留独立可回退提交，但未覆盖的合并验收项仍待验；不得为了交付批次合并而只做最后一次检查。

### 可操作验收场景

- **R13-09-V1**：同一外部ID重复行或冲突→保留原错误定位/冲突决议，不产生第二个实体。
- **R13-09-V2**：预检文件A后选择B→不得直接提交A确认；取消文件选择→不产生后台写入。
- **R13-09-V3**：commit中断后重新打开→先查询批次和结果，不盲目重复提交；数据与账本一致。

每个场景记录前置数据/身份、触发动作或延迟安排、预期状态/写入次数、实际结果、证据位置及验证层级。可控延迟/失败测试只模拟外部边界，必须运行真实工作流；持久化结论需要真实库读回。

### 现有检查入口与交付停止条件

以下命令从项目根目录执行，每条立即核对退出码；它们是现有守卫起点，不代表已经覆盖上述全部行为。实现时检查脚本实际断言，缺失场景补入现有测试/fixture并接入原验证入口，不创建同名空检查。

```powershell
node scripts/verify-monthly-workbooks.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-09: verify-monthly-workbooks failed' }
node scripts/verify-import-row-identity.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-09: verify-import-row-identity failed' }
node scripts/verify-team-package-localized-names.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-09: verify-team-package-localized-names failed' }
node scripts/verify-team-package-import-normalization.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-09: verify-team-package-import-normalization failed' }
node scripts/verify-match-review-package.mjs
if ($LASTEXITCODE -ne 0) { throw 'R13-09: verify-match-review-package failed' }
```

同时执行[共享验证门禁](../modular-rewrite/remaining-execution-contract.md)。任一业务场景失败、命令/Schema/保护资产非预期变化、仍存在双状态owner或无法说明的写入结果，均阻止该批交付。缺外部环境只按既定待验规则登记；R17-06和R18不得再延期必需真实验收。

**回退与交接：** 在修改前记录本批基线提交和用户已有修改；按执行约定的代码/数据双层回退处理。交接必须列出本批实际新owner、原函数/事件/类型去向、复用接口、逐项场景证据和剩余环境项。主交付以及所覆盖原ID只有满足各自义务才可记完成。

## 原节点去向（完整覆盖）

| 原节点 | 当前主交付 | 处理方式 |
|---|---|---|
| R13-01 Dashboard | R13-01 | 主交付 |
| R13-02 Database | R13-02 | 主交付 |
| R13-03 Competitions | R13-03 | 主交付 |
| R13-04 Rules | R13-03 | 合并验收项，已与03共同DONE |
| R13-05 Teams | R13-05 | 主交付 |
| R13-06 Players | R13-06 | 主交付 |
| R13-07 Lineup Presets | R13-07 | 主交付 |
| R13-08 Lineups | R13-07 | 合并验收项（非 DONE） |
| R13-09 Workbooks | R13-09 | 主交付 |
| R13-10 Architecture Page | R13-01 | 随01业务DONE；剩余设计内容由UI01承接 |

## 记录位置

阶段索引沿用 `docs/modular-rewrite/R13-core-ui-features/README.md`；记录以主交付 ID 命名，逐项列出原节点覆盖。现有索引已建立；已完成合并项沿原证据保持DONE，未完成项分别记待验，不能伪造完成。

## 阶段出口与交接

- 17页中分配给本阶段的页面/状态及原业务ID分别有关闭记录；布局补差与完整挂载验证随本域完成，不能默认推给R16/R17。仅既定真实外部环境项可登记到R17-06，设计或行为失败不属此豁免。
- 各主交付及其合并验收项均有记录，真实功能/状态/写入/错误和恢复路径接通；无空壳或伪造数据。
- 只退出已替代的职责，当前调用方、依赖、测试和清单已同步；不为满足文件名清单删除仍正确的共享实现。
- 所有本阶段适用门禁通过。最终新库待验按原安排逐项登记 owner、条件和 R17-06 接收位置；不得标记完整业务验收通过。
- 阶段启动时在既有记录体系建立阶段索引；每批记录覆盖原 ID，阶段结束创建阶段完成记录并附实际 A/M/R/D、契约、验证和回退摘要。记录路径沿用原阶段目录名，历史记录不重写。
- R14 仅在本阶段出口满足及用户授权后进入 READY。

## 已有设计输入的追溯

以下映射已按10/31审计订正，独立设计记录存在不等于生产已完成同状态验收。以[接入处置表](../modular-rewrite/ui-integration-disposition-20261031.md)确定复用、局部补差和待验责任；采用最新已批准设计，不要求保留已证实不匹配的旧布局。新增设计决定仍须Figma先行，已有真实能力必须保留。

适用[总纲第 8.0 节](00-总体架构与前23节.md#80-figma-设计接入依据2026-10-08-补充)。以下 S 编号及文件来自 `ui-design-system` 分支 `docs/`。以最新明确 accepted 记录与实际 Figma 节点为输入，复用 R12 的唯一组件实现。本节补充所有对应 Atomic Task 的设计输入和完成条件。

| R13 节点 | Figma 页面与记录 | 本次核对状态 / 对接要求 |
|---|---|---|
| R13-01 Dashboard | S01 `dashboard` / `FIGMA_SCREEN_DASHBOARD.md` | 视觉与代表性本页状态已验收；接入真实总览、连接状态和快捷入口 |
| R13-02 Database | S15 `database` / `FIGMA_SCREEN_DATABASE.md`及 `_STATE.json`；主Frame `673:178191` | 设计记录已存在；02业务DONE，概览/分区设计差异由UI01承接，真实重置仍沿BA-14 |
| R13-03 Competitions、04 Rules | S10 `rules` / `FIGMA_SCREEN_RULES.md`及 `_STATE.json` | 赛事目录、层级、路由、规则包属于同一设计路由，按代码职责拆分并保持页面入口一致 |
| R13-05 Teams | S06 `teams` / `FIGMA_SCREEN_TEAMS.md`及 `_STATE.json` | 目录、档案、名单、资料包与危险操作边界接入 |
| R13-06 Players | S07 `players` / `FIGMA_SCREEN_PLAYERS.md`及 `_STATE.json` | 目录、来源身份、履历、长期能力和短期标签分别接入 |
| R13-07 Lineup Presets | S08 `lineup_presets` / `FIGMA_SCREEN_LINEUP_PRESETS.md`及 `_STATE.json` | 预设编辑、归档与套用；预设和比赛副本分离 |
| R13-08 Lineups | S02 `lineups` / `FIGMA_SCREEN_LINEUPS.md`及 `_STATE.json` | 比赛、双方阵容、历史与局部工作包；保存和准入分开 |
| R13-09 Workbooks | S09 `workbooks` / `FIGMA_SCREEN_WORKBOOKS.md`及 `_STATE.json` | 三类工作包、预检、确认与批次隔离 |
| R13-10 Architecture Page | S17 `architecture` / `FIGMA_SCREEN_ARCHITECTURE.md`及 `_STATE.json`；主Frame `693:194738` | 设计记录已存在；原10随01业务DONE，能力/历史说明由UI01补差 |

“及 `_STATE.json`”指同名Markdown对应的独立状态文件。S15/S17已在设计分支有独立记录；实施时核对最新批准版本，不能继续引用旧检查点的未开始状态。

每页先将原有操作、当前接口和设计状态对齐，登记该页记录中的源码差异/运行时缺口，再在对应节点实现；不得因画板只覆盖代表性对象而删去其他合法数据路径。保留搜索/分页/选择、来源返回、错误不串用旧对象、失败草稿和工作区恢复等真实行为。数据变更及最终确认调用现有 API；Windows 验收复用已有 fixtures/脚本，真实数据库与工作簿待验仍按总纲登记。S15/S17当前是局部接入差异，归UI01；不重新安排已结束R12。
