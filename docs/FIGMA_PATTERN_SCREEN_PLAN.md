# Figma Patterns & Screen Plan

**2026-10-09 第二批修复检查点：INT-01 已修复；INT-03 的球队来源档案返回与 INT-02 的普通导航主题连续性已通过存储动作复验。705 个产品状态的普通主题变化候选为 0；28 个明暗来源档案保持首尔 FC 返回。整体仍为 NOT_PASSED，INT-04 待修复及全产品复验。未进行浏览器全事件回放或真实运行时验收。详见[修复记录](FIGMA_PROTOTYPE_INTEGRATION_REPAIRS.md)和[本批节点与复验数据](FIGMA_PROTOTYPE_REPAIR_BATCH02.json)。**

> 下文原验收的 933 根、561 状态、156 候选及“只读／尚未修复”均为历史快照；当前为 1,077 根、705 状态，INT-01/02/03 限定范围已修复，INT-04 未修复。当前状态以顶部、修复记录及第二批数据为准。

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互与数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支`ui-design-system`。

## 当前状态

图标、底层组件、六组Patterns以及计划内17条Screens的可编辑视觉与代表性本页状态阶段完成。Controls **61 sets/506 variants**；Patterns **17 sets/125 variants/13 production singles**。S17回读Foundations仍为**193 variables/13 text styles/6 effects**，没有修改共享资产。

**2026-10-09，Screens视觉与限定本页状态为17 / 17，没有尚未制作的产品页面。随后执行的全产品原型连通与状态一致性验收整体未通过，INT-01/02/03 的限定存储动作修复已完成，INT-04 待修复。** S01–S17 原视觉基线 561 状态，本批后为 705 状态，仍只计 17 条路由。弹窗、Review Hub、上下文菜单与Pattern QA不增加路由数。

**17页视觉完成不等于全产品集成、软件完成或发布通过。** 首次验收为只读；其后原型已修复接线并新增来源/主题状态，应用源码和共享资产未改，没有新增 S18。

最新恢复：[验收报告](FIGMA_PROTOTYPE_INTEGRATION_AUDIT.md) → [验收状态](FIGMA_PROTOTYPE_INTEGRATION_STATE.json) → [总状态](FIGMA_SCREENS_STATE.json)中的integrationRepair。S17[详细状态](FIGMA_SCREEN_ARCHITECTURE_STATE.json)和[独立记录](FIGMA_SCREEN_ARCHITECTURE.md)继续保留，完整页面清单见[17路由计划](FIGMA_SCREENS_PLAN.md)。

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件与Composer。

独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。共享API未改，不为进度更新重写历史组件记录；早期检查点不能覆盖后续Screens。

## Screens逐页视觉完成清单

- [x] S01 `dashboard`：总览、连接前置与代表性状态。
- [x] S02 `lineups`：比赛、双方阵容、链路、历史与局部工作包。
- [x] S03 `prediction`：正式推演、研究、历史与临时演练。
- [x] S04 `review`：九步复盘、手动补录、事件与候选。
- [x] S05 `runs`：运行列表、详情、追踪与历史隐藏。
- [x] S06 `teams`：目录、阵容、档案、任期/阵型与资料包。
- [x] S07 `players`：目录、来源身份、档案、能力/状态/标签与工作包。
- [x] S08 `lineup_presets`：球队预设、编辑校验、活动/归档与套用边界。
- [x] S09 `workbooks`：三类工作包、导出/预检/提交与批次隔离。
- [x] S10 `rules`：赛事目录、层级、路由与规则包。
- [x] S11 `release`：验收请求、分类检查、报告证据、性能/安全/成本与历史。
- [x] S12 `analytics`：历史、任务、质量、审核、回包与参数生命周期。
- [x] S13 `api_workspace`：普通文本、会话、只读上下文、请求与归档审计。
- [x] S14 `openai`：配置、密钥、示例解析、模型参数、测试与安全。
- [x] S15 `database`：连接与迁移、健康统计、清除本机连接与危险重置。
- [x] S16 `logs`：历史问题、重复聚合、技术详情、快照导出和清空边界。
- [x] S17 `architecture`：系统边界、运行原则、五类模块及静态阅读布局。
- [x] 各页独立状态明确记录的有限真实入站与返回。
- [x] 全产品存储原型图、默认导航矩阵与指定状态路径验收已执行，结论未通过。
- [x] INT-01/02/03 限定存储动作修复与复验。
- [ ] 修复 INT-04 并复验全产品导航、来源返回、主题与全局状态持续性。
- [ ] 全部真实功能、源码差异、文件/凭据/数据库/模型运行时与端到端集成。

各页历史计数、节点、源码差异及未实现事项以独立记录为准。S17只新增13个产品状态和5个只读弹窗，增加一条路由。

## 首次全产品原型验收（修复前历史）

933根及12,215项存储反应已检查；从首页不依赖Hub可到达17路由，各页Hub可达本页全部状态。失效目标和跨出Screens的导航0；3,738个可见Disabled实例的自身及祖先指针旁路0。

**阻断：** 34个默认明暗页面中92个非选中导航无有效点击、4个只显示说明；156条普通导航主题模式变化静态候选；S06来源球员档案切换S07基础资料后丢失来源返回入口；34个默认页面Back/Forward固定禁用，侧栏折叠和重置也只覆盖局部示例。具体数量、节点、已排除误报和复验条件以[报告](FIGMA_PROTOTYPE_INTEGRATION_AUDIT.md)与[状态](FIGMA_PROTOTYPE_INTEGRATION_STATE.json)为准。

这些是原型层集成结论，不把没有真实API或数据库当成视觉失败；也不把原型预置成功当成运行时通过。修复尚未开始，不在本轮改线或重做页面。

## S17历史验收与边界

四层职责是静态说明，不是正在运行的任务流程。只读标识与实际健康、运行版本、部署能力和发布验收分离；没有数据库也可阅读。隐藏从旧Shell继承的配置徽章，未取得运行版本仍显示“版本未提供”。

正常业务修订保留历史不意味着记录永不删除或已备份。路由自动选择有启用、时间、上下文和模型筛选前提，没有匹配明确报错；路由匹配、输入就绪、外部模型可用和正式许可分别判断。接口与stub不表示已安装外部计算实现，模块路径不是完整运行时清单。

最终**165 NAVIGATE +66 OVERLAY =231，10 CLOSE，0 SWAP及其他动作**。Hub `694:197834`可达13状态和5弹窗；6条S15/S16真实入站另计。默认Light/Dark保留代表性主题，无数据库/恢复失败进入本地只读说明，不转移凭据、日志、修复对象或权限。

2484未隐藏节点、900实例、598文字层检查通过。非预期越界、失效引用/目标、未绑定颜色、顶层重叠、异常按钮及静态断言错误0；52个禁用控件反应与祖先旁路0。18处局部文字对比度修正后最低Light4.5327/Dark6.8625。5处实际超高阅读区域，底部验收栏外置。

本页没有最终写操作按钮，五个弹窗只有解释与关闭，没有实际网络、模型、文件、凭据或数据库调用。Compact全局主题及Shell折叠/重置等未覆盖控制只显示说明，不宣称完整响应式行为或全状态持续性。

## 历史保留与后续实现

S17没有修改旧页视觉、共享组件或应用源码，只补6条原型入站。本轮只读验收不修改各页历史记录，也不重新检查全部视觉/对比度或运行时实现。

继续保留：S16历史记录非当前故障、日志清空不修复业务且内存/文件失败需核验；S15清空schema事务先提交后重建、失败不保证回滚；S14配置选中/当前/密钥/测试独立与旧响应失效；S13主动勾选只读上下文及session/request/草稿隔离；S12精确分区许可与候选/历史/正式参数分离；S11报告完成非发布；S10读取/校验/注册/绑定/执行独立；S09文件/模式/批次隔离；S08预设与比赛副本；S07长期能力/短期标签与来源身份；S06归档/删除/强清及导入/P4分开；S05隐藏不删血缘；S04查看/执行与复盘/结算；S03输入/提供器/输出；S02双方11首发与失败草稿；Workflow和Timeline原有语义。

各页已经记录的真实参数输入、来源身份、异步竞态、幂等与结果未知恢复、Windows凭据、文件校验、数据库事务与恢复、模型可用性、日志脱敏和完整主题/滚动/焦点/草稿持续性待办都保留。S17静态说明今后也应随实现更新，不能作为永久运行安全保证。

**下一步为修复原型集成问题并复验，不是新增页面。原型修复与真实功能实现均尚未启动。**

## 恢复

读取README、验收报告、验收状态、最新Screens总状态中的integrationAudit及实际Figma稳定节点。S17视觉基线`67e2c5000dce035b70860826c55ccde61fe84396`；S16完整检查点`7d3b72310a745a007075d3b8ed35ef38cc33a49d`；S15`920432210478b26e17f2a3c9ffaf74c0f9f01e6f`；S14`52236b78b5a0f478bb6fb3b4235d30fd92d4e97c`；更早链保留在Screens计划与Git历史。

早期WIP、旧截图、空handoff和旧“未开始”不能覆盖最新记录。各页accepted只表示其限定局部范围，不覆盖本次整体验收未通过结论。SLOT后代按稳定根、语义名称和真实主组件定位，不猜ID。Create State handoff未创建；以GitHub独立记录和同步总检查点恢复。
