# Figma Patterns & Screen Plan

**设计依据（2026-08-28确认）：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互和数据语义。旧CSS几何不是新设计依据。**

## 当前状态

截至 **2026-10-01**，图标、底层组件及Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有9 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks。S10–S17共8条未开始。** 累计280个产品状态仍只计9条路由；弹窗、上下文菜单、Review Hub和Pattern QA不增加路由数量。

恢复：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S09详细状态](FIGMA_SCREEN_WORKBOOKS_STATE.json) → [S09记录](FIGMA_SCREEN_WORKBOOKS.md)。顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S10 `rules` / 规则与模型，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件与Composer。

各组独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat历史状态](FIGMA_AI_CHAT_STATE.json)及[Workflow历史状态](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)不覆盖后续Screens进度。

## 当前阶段：Screens

- [x] S01 `dashboard`：总览、连接前置及代表性状态。
- [x] S02 `lineups`：比赛、双方阵容、模型链路、历史和局部工作包。
- [x] S03 `prediction`：正式推演、研究、历史和临时演练。
- [x] S04 `review`：九步复盘、补录、事件与候选。
- [x] S05 `runs`：运行列表、详情、技术追踪与历史隐藏。
- [x] S06 `teams`：目录、名单、档案、阵型/任期及资料包。
- [x] S07 `players`：目录、来源身份、档案、履历、状态/能力/标签和工作包。
- [x] S08 `lineup_presets`：按球队管理预设、编辑校验、操作确认及套用边界。
- [x] S09 `workbooks`：三类导出/预检/提交、独立批次、规则和结果保护。
- [x] 各独立页状态中列明的部分真实入站与返回。
- [ ] **S10 `rules`：下一项，尚未开始。**
- [ ] S11–S17：见Screens计划，均未开始。
- [ ] 其他真实目标、全状态持续性、完整跨路由和运行时联调。

S01–S08历史计数与验收含义保留。S09新增30状态、26弹窗及Review Hub `582:112939`，仍只增加一条路由。

S09最终读回 **585条NAVIGATE/OVERLAY、78项CLOSE、0其他动作**；30状态及26弹窗可达，7条外部入站另计。3,186个未隐藏实例可解析；非预期越界、失效引用/目标、未绑定solid颜色、异常按钮高度和顶层重叠0。167个禁用控件及其可点击父级检查通过；17处正常滚动。普通文字最低对比度Light4.5327/Dark6.8625。

六条S06/S07/S08工作包导航进入对应主题通用入口；S07球员工作包的“全部工作包”仅选择球员类别。没有转移此前文件、模式或批次。既有其他页视觉、数据和验证规则未改。

## 页面与实现边界

使用真实实例，不detach、不创建重复Screen主组件、不以旧QA替代产品目标。S09只改自有节点和7条旧页入口，未改应用源码、共享变量、样式和主组件。

三类文件和预检结果独立。Team Ready → Player Isolated → Team Ready是有限上下文保留示例；其他多批次组合只读说明。仅新增重新预检为T-DEMO-02，不直接提交T-DEMO-01。空白保留、clear显式清空且仍校验，未知不补0。

冲突和格式错误独立；无变更、忙、已导入/重复、文件/模式变化、结果不明禁提交。结果未确认先核验、不重发；Imported表格标为原预检快照。已有批次重新选文件不经演示选择器绕回旧Ready。

10个最终操作确认（6导出、4导入）无执行连接，成功/错误由目录独立选择；批次明细是结构说明，不是完整真实JSON。没有执行文件读取、导出、预检、冲突修改或数据库写入。

**源码待办：** 当前workbooks.ts ready表达式只检查preview存在及冲突/错误。本设计的附加状态保护仍须实现阶段核对控制器/后端并落实，不把视觉通过写成源码已修复，也不声称其他层均无保护。

S08同队、11首发、唯一阵型槽位与概率校验保持，套用只覆盖目标侧草稿；归档/永久删除各有范围。S07有效履历、可用性、长期能力和动态标签分开；S06导入/P4评分独立，归档/普通删除/强制清除分开；S05隐藏不删血缘；S04查看/执行、复盘/结算分离，SHA变化拒绝旧确认；Timeline修订/核验/取消独立；S03输入/提供器/输出独立；S02双方各11首发且失败保留草稿；AI Chat只读上下文主动勾选及session/request隔离。

所有实体、版本、文件、批次和结果为预置示例。任意输入、三类并发批次、全部导出范围/覆盖确认、真实解析、事务/幂等、权限、模型/Worker、焦点/竞态及完整主题/滚动/草稿持续性均需后续实现。有限原型不代表所有控件可执行或17路由端到端通过。

## 恢复

读取最新Screens计划、总状态、当前页详细状态与实际Figma稳定根。S08历史总状态 `06eee1a492a92b620485d194cdd83844ecadef13`；S07 `66d575f0d9985012a43448e2fed2af9f913c3117`；更早版本见Screens计划。

S09初始与组合WIP由accepted替代，不因旧截图或中途失败重复创建。Create State没有新handoff；恢复依赖GitHub明确状态。未变组件API不为更新进度重写；SLOT虚拟ID按稳定根、语义名称和真实实例引用恢复，不猜ID。
