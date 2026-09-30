# Figma Patterns & Screen Plan

**设计依据（2026-08-28确认）：Figma决定视觉尺寸、比例、间距、字号和颜色；应用源码只提供功能、交互和数据语义。旧源码尺寸是历史取证，不是新设计依据。**

## 当前状态

截至 **2026-09-30**，图标、既定底层组件和Patterns六组均完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已完成3 / 17条路由的视觉、本页适用状态及有限原型：S01 dashboard、S02 lineups、S03 prediction。S04–S17共14条未开始。** 状态页、对话框、Review Hub和Pattern QA不增加路由计数。

当前恢复入口：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S03详细状态](FIGMA_SCREEN_PREDICTION_STATE.json) → [S03记录](FIGMA_SCREEN_PREDICTION.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S04 `review` / 赛后复盘，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级导航、二级导航、Topbar、折叠与主题保持。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector工作区（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件、Composer。

依赖按组归档，不合并成一份大文档。证据：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)、[Master–Detail–Inspector](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow / Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat检查点](FIGMA_AI_CHAT_STATE.json)和[Workflow检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)是历史记录，不能覆盖更新后的Screens进度。

## 当前阶段：Screens

- [x] S01 `dashboard`：数据总览、连接前置及本页状态。
- [x] S02 `lineups`：比赛、双方阵容、模型链路、历史、页内工作包。
- [x] S03 `prediction`：正式推演、P4研究与收敛、最近历史、临时演练。
- [x] S01默认Light/Dark到真实S02比赛入口，以及S03新增7条S01/S02入站。
- [ ] **S04 `review`：下一项，未开始。**
- [ ] S05–S17：见独立Screens计划，均未开始。
- [ ] 剩余真实路由接线、全状态持续性与完整跨路由联调。

S01原验收15状态，S02原验收33状态/7对话框。S03本轮33状态/3对话框，323条NAVIGATE/OVERLAY、9项CLOSE；7条外部入站单独计数。S03研究和最近历史不代表后续独立路由完成。

S03最终深层检查3,339个未隐藏实例、2,057个普通文字层，非预期越界、失效引用、未绑定solid颜色、顶层重叠均0；22处正常滚动单列。257个Disabled控件无反应。普通文字最低对比度Light4.53:1/Dark6.86:1。具体节点与条件示例边界见S03记录和状态文件。

历史S01/S02记录继续保留；其原先“待接线”及原计数描述对应当次验收快照，最新集成增量以Screens总状态为准。没有重写未变化的组件API记录。

## 页面与实现边界

页面使用已完成组件的真实实例；不detach、不创建重复Screen主组件、不用旧QA作为产品目标。本轮没有修改应用源码、全局变量或既有主组件。

S03确认当前源码未捆绑真实P4/P7模型：输入门禁、提供器可用与结果返回独立。概率结果只是明确标记的条件示例，不是模型运行。正式阻断不能借影子入口绕过；窗口变化使旧指纹和许可失效；S02链路Blocked入口保持Disabled。

研究人工决策只追加、不覆盖来源；采用来源只按较可信状态参与路由；接受未知不等于0；截止后只读，冻结快照不可变。临时演练不保存比赛，历史移除不删除运行血缘，最终写操作不连接到伪造成功。

既有约束仍保留：S02双方各11名首发才能提交、失败保留草稿、保存与模型准入独立、导入先预检后确认。Workflow查看步骤与执行权限独立；Timeline核验/修订/取消与有效比分由数据驱动；AI Chat纯文本、只读上下文主动勾选，Pending/取消绑定sessionId/requestId，附件不扩展为上传、生成文件或数据库执行。

所有预置状态、人工演示按钮和有限导航都不代替真实API、数据库、模型、任务Worker、权限、输入、键盘焦点、异步竞态以及主题/草稿/滚动持续性测试。完整17路由集成尚未完成，不声称所有控件已交互实现。

恢复时读取最新Screens计划、总状态、当前页详细状态与实际节点；旧截图、组件阶段Screens未开始文字、空handoff标题均不得覆盖明确新检查点。
