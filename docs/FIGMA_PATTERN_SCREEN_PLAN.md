# Figma Patterns & Screen Plan

**设计依据（2026-08-28确认）：Figma决定视觉尺寸、比例、间距、字号和颜色；应用源码只提供功能、交互和数据语义。旧源码尺寸是历史取证，不是新设计依据。**

## 当前状态

截至 **2026-09-30**，图标、既定底层组件和Patterns六组均完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有4 / 17条路由完成可编辑视觉、代表性本页状态与有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review。S05–S17共13条尚未开始。** 114个累计状态画面仍只计4条路由；弹窗、Review Hub与Pattern QA不增加页面数。

当前恢复入口：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S04详细状态](FIGMA_SCREEN_REVIEW_STATE.json) → [S04记录](FIGMA_SCREEN_REVIEW.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S05 `runs` / 运行记录，尚未开始。**

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
- [x] S04 `review`：九步复盘、手动补录、结果事件、能力候选和最近复盘的代表性状态。
- [x] 选定S01→S02、S01/S02→S03、S01/S02/S03→S04入站及明确产品返回。
- [ ] **S05 `runs`：下一项，尚未开始。**
- [ ] S06–S17：见独立Screens计划，均未开始。
- [ ] 剩余真实目标、全状态持续性、完整跨路由和运行时联调。

S01原验收15状态，S02原验收33状态/7弹窗，S03原验收33状态/3弹窗；S04本轮33状态/18弹窗，其中9个弹窗为只读步骤说明。原记录计数对应各次验收快照，后续增量由最新状态记录，不回写未改变的组件历史。

S04最后读回531条NAVIGATE/OVERLAY、54项CLOSE与93项其他组件/继承动作；6条外部入站单列。33状态与18弹窗均从 `509:48662` 可达。5,246个未隐藏实例可解析；非预期越界、未绑定solid颜色、顶层重叠、异常按钮高度均0；10处正常滚动单列；256个Disabled控件无反应；普通文字最低对比度Light4.5327/Dark6.8625。

六条S04入站均到 `505:34022` 未选择比赛，不擅自沿用源页面比赛、资料包、主题或权限。代表性Dark/Compact另行验证，不把这几条导航解释成完整运行时持续性。

## 页面与实现边界

页面继续使用已完成组件真实实例；不detach、不创建重复Screen主组件、不把旧QA当产品目标。S04没有修改应用源码、全局变量或既有主组件。

S04查看步骤与allowed_actions权限分离；第4步阻断时回看第2/8步仍保留第4步Blocked。文件SHA变化使旧人工确认失效；预检、确认、写事实、生成复盘、正式结算分别判断。结算依赖实际模型运行、不可变快照、赛事Profile及正式时点，不因复盘生成而自动允许。候选仍需单独审核，接受后才允许后端回写正式能力历史。

Timeline保留取消事件审计，不累加到有效比分；修订不升级核验。手动补录空/非法/缺阵容受控，合法任意提交及全部补充量化字段交互未作为本轮完成内容。六个写操作最终确认不连接到伪造成功。

S03继续保持输入门禁、提供器可用与结果返回独立，公开源码不捆绑实际P4/P7算法；研究决策只追加、截止后只读、未知不等于0、冻结快照不可变。S02保持双方各11名首发、失败保留草稿、保存与模型准入独立以及先预检后确认。

AI Chat保持纯文本、只读上下文主动勾选、Pending/取消绑定sessionId/requestId；不扩展为上传、生成文件或数据库执行。

所有状态、文件、比分、任务、概率、结算和候选都是预置演示。真实API、数据库事务、SHA校验、文件、权限、模型、Worker、输入、键盘焦点、异步竞态以及完整主题/草稿/滚动持续性须后续测试。有限原型未声称所有控件均可交互或17条路由端到端通过。

## 恢复

读取最新Screens计划、总状态、当前页详细状态和实际Figma稳定根；历史S03总状态在 `ff60437ccd38e1ed169cbebad0b21f7d8610e128`。当前S04先前分配/组合WIP已被accepted检查点替代。

组件总记录中的Screens未开始、旧截图、空handoff和旧计数不得覆盖新明确检查点。未变化的组件API不重写；规范化SLOT虚拟ID按稳定根、语义名称和实际引用恢复。
