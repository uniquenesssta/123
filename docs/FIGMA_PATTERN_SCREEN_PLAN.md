# Figma Patterns & Screen Plan

**设计依据（2026-08-28确认）：Figma决定视觉尺寸、比例、间距、字号和颜色；源码只提供功能、交互和数据语义。旧CSS几何不是新设计依据。**

## 当前状态

截至 **2026-10-01**，图标、底层组件和Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有7 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players。S08–S17共10条未开始。** 累计219个产品状态仍只计7条路由，弹窗、上下文菜单、Review Hub、Pattern QA不增加路由数量。

恢复：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S07详细状态](FIGMA_SCREEN_PLAYERS_STATE.json) → [S07记录](FIGMA_SCREEN_PLAYERS.md)。顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S08 `lineup_presets` / 阵容预设，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史、消息、只读附件和Composer。

各组独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat状态](FIGMA_AI_CHAT_STATE.json)和[Workflow状态](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)只代表历史，不覆盖后续Screens进度。

## 当前阶段：Screens

- [x] S01 `dashboard`：数据总览、连接前置和代表状态。
- [x] S02 `lineups`：比赛、双方阵容、模型链路、历史、页内工作包。
- [x] S03 `prediction`：正式推演、P4研究、最近历史、临时演练。
- [x] S04 `review`：九步复盘、手动补录、结果事件、候选和最近复盘。
- [x] S05 `runs`：独立列表、详情、技术追踪、历史隐藏及连接前置。
- [x] S06 `teams`：目录、阵容、速览、代表档案、任期/阵型、完整资料包、新增。
- [x] S07 `players`：目录、来源球队、速览、A01档案、履历/可用性/能力/标签、球员工作包和新增。
- [x] 既有页面选定入站，以及S06→S07四条通用目录、一条A01档案入口及明确球队返回。
- [ ] **S08 `lineup_presets`：下一项，尚未开始。**
- [ ] S09–S17：见独立Screens计划，均未开始。
- [ ] 其他真实目标、全状态持续性、完整跨路由和运行时联调。

S01–S06历史状态计数15、33、33、33、18、42保持当次含义。S07增加45状态/30弹窗/1Review Hub `557:87148`，仍只增加一条路由；18只读摘要和12写确认不是独立页面。

## S07验收与集成增量

最终读回925条NAVIGATE/OVERLAY、90项CLOSE、0其他动作；45状态/30弹窗全部可达。五条S06外部入站单列：四条Page/players进入Light/Dark通用目录；A01速览进入 `550:73700`，两个返回按钮均到真实首尔FC档案 `527:57101`。S06只另更新一处接线说明，无S01新增入口。

深层13,520个未隐藏节点、5,575嵌套实例，引用全部解析。非预期越界、未绑定可见颜色、顶层重叠、异常按钮高度及失效原型目标0；265禁用/忙控件自身反应和可点击父容器旁路均0。13处正常滚动；3,493个普通文字层最低对比度Light4.5048/Dark5.5998。

截图修正MDI虽然计算尺寸正确但仍使用旧示例裁切边界的问题。当前实际屏幕实例/SLOT显式尺寸与非滚动裁切已修复，完整操作列与分页可见；档案和导入操作栏保持在滚动区外。全部变化局限当前实例覆盖，没有改共享主组件。

## 页面与实现边界

使用现有真实组件，不detach、不创建重复Screen主组件、不以旧QA替代产品目标。S07没有修改应用源码、共享变量和样式；旧页面仅五条S06入口和一处说明改变。

当前打开球员与选择集合独立。A02加载失败不展示A01旧载荷；非A01档案保持各自只读摘要。A01编辑为代表，不等于全部球员任意编辑已实现。

生命周期、当前可用性、有效履历、历史观察和短期标签分开；过期/未来标签不算当前效果，未知值不补成0/默认50。贡献入口只说明范围，不执行算法。源身高编辑120–230、新增100–240的差异保留为运行时校验待办。

球员工作包不能误收球队月度文件；预检不写库；冲突、格式错误、无待处理项、写入中与结果不明分别禁用。归档保留历史；删除只允许无引用C01并保护A01，没有球员强制清除入口。12个最终写确认反应均0；成功/失败为评审目录独立示例。

S06原A/B身份分离、归档/普通删除/强制清除边界及导入/P4资格继续保留；S05隐藏不删血缘；S04查看/执行/结算分离与候选独立审核；Timeline核验/修订/取消独立；S03输入/提供器/输出与冻结证据分开；S02双方各11首发；AI Chat只读上下文主动勾选、sessionId/requestId隔离。

全部实体、日期、文件、评分和结果为预置。任意输入、多选、所有对象载荷、真实文件/数据库、后端权限、模型/Worker、结算、焦点、竞态及完整主题/来源/草稿/滚动持续性须后续实现。代表性A01返回路径不是全部比赛/球队来源贯通；球员工作包不完成S09。有限原型不声明所有控件可交互或17路由端到端通过。

## 恢复

读取最新Screens计划、总状态、S07独立状态和实际Figma稳定根。S06历史总检查点 `ac85c1b42be55bd7378b0990a3406b6da4d9d087`；S05 `444ad13d6e2de9fe9d99eb9a3c15022b40cd2ef4`；S04 `1d2b9b78d4d7bf2e589c81e69e4fef89638c98cb`；S03 `ff60437ccd38e1ed169cbebad0b21f7d8610e128`。S07早期WIP已被accepted取代。

旧截图、空handoff、旧WIP或组件阶段Screens未开始文字不能覆盖最新明确状态。组件API不为更新进度重写；规范化SLOT虚拟ID按稳定根、语义名称及实际组件引用恢复，不猜ID。
