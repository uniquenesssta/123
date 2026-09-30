# Figma Patterns & Screen Plan

**设计依据（2026-08-28确认）：Figma决定视觉尺寸、比例、间距、字号和颜色；应用源码只提供功能、交互和数据语义。旧源码尺寸是历史取证，不是新设计依据。**

## 当前状态

截至 **2026-09-30**，图标、底层组件和Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有6 / 17条路由通过可编辑视觉、代表性本页状态与有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams。S07–S17共11条未开始。** 累计174个产品状态仍只计6条路由；弹窗、上下文菜单、Review Hub、Pattern QA不增加页面计数。

恢复入口：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S06详细状态](FIGMA_SCREEN_TEAMS_STATE.json) → [S06独立记录](FIGMA_SCREEN_TEAMS.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S07 `players` / 球员，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级导航、二级导航、Topbar、折叠与主题保持。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector工作区（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件、Composer。

各组独立归档，不合并为大文档。证据：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat检查点](FIGMA_AI_CHAT_STATE.json)和[Workflow检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)只代表历史，不覆盖后续Screens进度。

## 当前阶段：Screens

- [x] S01 `dashboard`：数据总览、连接前置及代表性本页状态。
- [x] S02 `lineups`：比赛、双方阵容、模型链路、历史、页内工作包。
- [x] S03 `prediction`：正式推演、P4研究、最近历史、临时演练。
- [x] S04 `review`：九步复盘、手动补录、结果事件、候选和最近复盘。
- [x] S05 `runs`：独立运行列表、详情、技术追踪、历史隐藏和连接前置。
- [x] S06 `teams`：目录、阵容、速览、代表性完整档案、任期/阵型、完整资料包、新增资料。
- [x] 选定S01→S02、S01/S02→S03、S01/S02/S03→S04、S02/S03/S04→S05及S01–S05资源导航→S06入口。
- [ ] **S07 `players`：下一项，尚未开始。**
- [ ] S08–S17：见独立Screens计划，均未开始。
- [ ] 其余真实目标、全状态持续性、完整跨路由及运行时联调。

历史S01–S05状态数量15、33、33、33、18保留各自验收含义。S06增加42状态/27弹窗/1Review Hub `537:69771`，仍只增加一条路由。S06球员速览、预设摘要、资料包不算后续独立页面完成。

S06最后读回658条NAVIGATE/OVERLAY、77项CLOSE、0其他动作；10条外部入站另计。42状态、27弹窗均可达。4,723个未隐藏实例可解析，非预期越界、未绑定可见颜色、失效目标、顶层重叠和异常按钮高度0；8处正常滚动。256个禁用/忙控件无反应，禁用控件可点击父容器旁路0。普通文字最低对比度Light4.5048/Dark6.6589。

十条新入站来自S01–S05默认Light/Dark的Module/resources，分别进入S06对应主题目录，不自动沿用任意球队身份、筛选或草稿。其他状态的主题、滚动、折叠持续性没有因此完成。

## 页面与实现边界

页面使用现有组件真实实例，不detach、不创建重复Screen主组件、不以旧QA替代产品目标。S06没有修改源码、共享变量、样式或既有主组件，只修改本页自有节点/实例覆盖及10个旧页面入口反应。

S06批量选择不等于当前打开对象；A/B名单不串线，B加载/失败不展示A字段。A完整编辑为代表，B–F使用身份匹配的只读摘要；A01速览不代表所有球员详情已实现。中文名空白保留、教练任期投影、观察次数与联合范围校验分开。

归档保留历史；普通永久删除仅允许无引用C，A受保护；强制清除按球队级预检影响和后端confirmation_text再次确认，不是整库清除。17个写相关最终确认均无执行连接。

完整资料包结构/待处理条件与P4输入评分分开。65分无阻断且有待处理行可导入，仍低于P4门槛；ready=0即使高分也禁导入。文件冲突与其他格式错误分别处理，写入中禁重复，错误先核验结果。文件选择和结果状态仅为预置，不执行真实导入。

S05隐藏不删运行血缘，未知概率不补0；S04查看与allowed_actions分离、SHA变化拒绝旧确认、复盘生成与结算分开、候选接受才允许回写；Timeline修订不升级核验，取消不累计比分。S04合法任意手动提交和补充字段依然是实现待办。

S03输入门禁、独立ModelProvider、实际输出分开，研究决策追加、截止后只读、冻结不可变；S02双方11名首发、失败保留草稿、保存与准入独立；AI Chat纯文本、只读上下文主动勾选、sessionId/requestId隔离，不扩展上传或数据库执行。

所有实体、日期、文件、评分、概率和结果均为示例。任意输入、多选、全部球员/球队载荷、真实文件、数据库事务、权限、模型/Worker、结算、键盘焦点、竞态以及完整主题/滚动/草稿持续性未计为本轮完成。有限原型不是所有控件交互或17路由端到端验收。

## 恢复

读取最新Screens计划、总状态、当前页详细状态和实际Figma稳定根。S05历史总状态为 `444ad13d6e2de9fe9d99eb9a3c15022b40cd2ef4`；S04为 `1d2b9b78d4d7bf2e589c81e69e4fef89638c98cb`；S03为 `ff60437ccd38e1ed169cbebad0b21f7d8610e128`。S06早期分配/组合WIP已被accepted替代。

旧截图、空handoff、组件阶段Screens未开始或旧计数不能覆盖最新检查点。组件API不为更新进度而重写；规范化SLOT虚拟ID按稳定根、语义名称及真实实例引用恢复，不猜ID。
