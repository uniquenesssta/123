# Figma Patterns & Screen Plan

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互与数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支`ui-design-system`。

## 当前状态

图标、底层组件、六组Patterns以及计划内17条Screens的可编辑视觉与代表性本页状态阶段完成。Controls **61 sets/506 variants**；Patterns **17 sets/125 variants/13 production singles**。S17回读Foundations仍为**193 variables/13 text styles/6 effects**，没有修改共享资产。

**2026-10-09，S17 architecture / 架构信息已验收，Screens视觉与限定本页状态为17 / 17，没有尚未制作的产品页面。** S01–S17累计561个产品状态仍只计17条路由。弹窗、Review Hub、上下文菜单与Pattern QA不增加路由数。

**全产品导航和状态持续性、真实运行时及端到端验收仍未完成。17页视觉完成不等于软件完成或发布通过。** 没有擅自新增S18或启动后续实现工作。

恢复：[总状态](FIGMA_SCREENS_STATE.json) → [S17详细状态](FIGMA_SCREEN_ARCHITECTURE_STATE.json) → [S17独立记录](FIGMA_SCREEN_ARCHITECTURE.md)。完整页面清单与后续边界见[17路由计划](FIGMA_SCREENS_PLAN.md)。

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
- [ ] 单独核对全产品导航、所有目标与身份、主题/折叠/返回/滚动/焦点/草稿持续性。
- [ ] 全部真实功能、源码差异、文件/凭据/数据库/模型运行时与端到端集成。

各页历史计数、节点、源码差异及未实现事项以独立记录为准。S17只新增13个产品状态和5个只读弹窗，增加一条路由。

## S17验收与边界

四层职责是静态说明，不是正在运行的任务流程。只读标识与实际健康、运行版本、部署能力和发布验收分离；没有数据库也可阅读。隐藏从旧Shell继承的配置徽章，未取得运行版本仍显示“版本未提供”。

正常业务修订保留历史不意味着记录永不删除或已备份。路由自动选择有启用、时间、上下文和模型筛选前提，没有匹配明确报错；路由匹配、输入就绪、外部模型可用和正式许可分别判断。接口与stub不表示已安装外部计算实现，模块路径不是完整运行时清单。

最终**165 NAVIGATE +66 OVERLAY =231，10 CLOSE，0 SWAP及其他动作**。Hub `694:197834`可达13状态和5弹窗；6条S15/S16真实入站另计。默认Light/Dark保留代表性主题，无数据库/恢复失败进入本地只读说明，不转移凭据、日志、修复对象或权限。

2484未隐藏节点、900实例、598文字层检查通过。非预期越界、失效引用/目标、未绑定颜色、顶层重叠、异常按钮及静态断言错误0；52个禁用控件反应与祖先旁路0。18处局部文字对比度修正后最低Light4.5327/Dark6.8625。5处实际超高阅读区域，底部验收栏外置。

本页没有最终写操作按钮，五个弹窗只有解释与关闭，没有实际网络、模型、文件、凭据或数据库调用。Compact全局主题及Shell折叠/重置等未覆盖控制只显示说明，不宣称完整响应式行为或全状态持续性。

## 历史保留与后续实现

S17没有修改旧页视觉、共享组件或应用源码，只补6条原型入站。17个默认根存在性已确认，不重新验收S01–S16，也不把存在性等同全产品集成。

继续保留：S16历史记录非当前故障、日志清空不修复业务且内存/文件失败需核验；S15清空schema事务先提交后重建、失败不保证回滚；S14配置选中/当前/密钥/测试独立与旧响应失效；S13主动勾选只读上下文及session/request/草稿隔离；S12精确分区许可与候选/历史/正式参数分离；S11报告完成非发布；S10读取/校验/注册/绑定/执行独立；S09文件/模式/批次隔离；S08预设与比赛副本；S07长期能力/短期标签与来源身份；S06归档/删除/强清及导入/P4分开；S05隐藏不删血缘；S04查看/执行与复盘/结算；S03输入/提供器/输出；S02双方11首发与失败草稿；Workflow和Timeline原有语义。

各页已经记录的真实参数输入、来源身份、异步竞态、幂等与结果未知恢复、Windows凭据、文件校验、数据库事务与恢复、模型可用性、日志脱敏和完整主题/滚动/焦点/草稿持续性待办都保留。S17静态说明今后也应随实现更新，不能作为永久运行安全保证。

**下一步是独立的全产品导航与状态持续性验收／真实实现，不是新的未制作页面。上述工作本轮尚未启动。**

## 恢复

读取README、最新Screens计划、总状态、S17详细状态、独立记录与实际Figma稳定根。S16完整检查点`7d3b72310a745a007075d3b8ed35ef38cc33a49d`；S15`920432210478b26e17f2a3c9ffaf74c0f9f01e6f`；S14`52236b78b5a0f478bb6fb3b4235d30fd92d4e97c`；更早链保留在Screens计划与Git历史。

早期WIP、旧截图、空handoff和旧“未开始”不能覆盖最新accepted。SLOT后代按稳定根、语义名称和真实主组件定位，不猜ID。Create State handoff未创建；以GitHub独立记录和同步总检查点恢复。
