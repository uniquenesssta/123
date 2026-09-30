# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并成一份大记录。

**设计依据（2026-08-28 用户确认）：视觉尺寸、比例、间距、字号和颜色均以已确认的Figma为准。应用源码只用于功能入口、交互和数据语义，不作为视觉尺寸参考；旧源码尺寸只作历史取证。**

## 当前恢复入口

**2026-09-30最新检查点：S04 review / 赛后复盘的视觉、代表性本页状态及有限原型已验收。** Screens当前 **4 / 17 条产品路由**：S01 dashboard、S02 lineups、S03 prediction、S04 review；S05–S17共13条尚未开始。

- [Screens总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S04赛后复盘：九步、事实、结算和候选边界](FIGMA_SCREEN_REVIEW.md)
- [S04明确恢复状态与节点账本](FIGMA_SCREEN_REVIEW_STATE.json)
- [S03赛事推演：原验收记录](FIGMA_SCREEN_PREDICTION.md)
- [S03独立恢复状态](FIGMA_SCREEN_PREDICTION_STATE.json)
- [S02比赛与阵容：原验收记录](FIGMA_SCREEN_LINEUPS.md)
- [S02独立恢复状态](FIGMA_SCREEN_LINEUPS_STATE.json)
- [S01 Dashboard：原验收记录](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S05 `runs` / 运行记录，尚未开始。** S04有33个状态、18个弹窗和1个Review Hub；18个弹窗中的9个是只读步骤说明，均不增加产品路由数量。S03最近运行和S04最近复盘不代表S05完成。

## 组件与模式记录

- [组件阶段记录与Figma节点](FIGMA_COMPONENT_RECORD.md)
- [按钮、Spinner与Loading组件](FIGMA_BUTTON_COMPONENTS.md)
- [扩展字段组件](FIGMA_EXTENDED_FIELDS.md)
- [Searchable Combobox组件](FIGMA_SEARCHABLE_COMBOBOX.md)
- [Switch / Toggle组件](FIGMA_SWITCH_TOGGLE.md)
- [Tabs / Segmented Control组件](FIGMA_TABS_SEGMENTED_CONTROL.md)
- [Data Table / Pagination组件](FIGMA_DATA_TABLE_PAGINATION.md)
- [Dropdown / Context / Overflow Menu组件](FIGMA_DROPDOWN_CONTEXT_OVERFLOW_MENU.md)
- [Dialog：普通、确认与名称校验危险确认](FIGMA_DIALOG.md)
- [Toast / Inline Alert / Blocking Message与Task Activity](FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md)
- [Accordion / Disclosure与展开状态恢复](FIGMA_ACCORDION_DISCLOSURE.md)
- [Progress / Skeleton / Empty State与异步状态边界](FIGMA_PROGRESS_SKELETON_EMPTY_STATE.md)
- [Avatar：球队、球员与默认占位](FIGMA_AVATAR.md)
- [PC App Shell：导航、顶栏与交互验收](FIGMA_PC_APP_SHELL.md)
- [Page Heading / Toolbar / Filter Bar / Selection Command Bar](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)
- [Metric Card / Action Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)
- [Master–Detail–Inspector与Entity Row](FIGMA_MASTER_DETAIL_INSPECTOR.md)
- [Workflow Stepper / Timeline与交互验收](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)
- [Workflow / Timeline历史检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)
- [AI Chat：历史、消息、只读附件与Composer](FIGMA_AI_CHAT.md)
- [AI Chat历史检查点与节点账本](FIGMA_AI_CHAT_STATE.json)
- [应用图标待办](FIGMA_ICON_BACKLOG.md)
- [基础Token与底层组件待办](FIGMA_FOUNDATION_COMPONENT_BACKLOG.md)
- [Patterns与Screens总阶段计划](FIGMA_PATTERN_SCREEN_PLAN.md)

执行顺序：图标 → 基础依赖与组件 → Patterns → Screens。

图标、既定底层组件及Patterns六组全部完成。Controls保持 **61 sets /506 variants**，Patterns保持 **17 sets /125 variants /13 production singles**，Foundations保持 **193 variables /13 text styles /6 effects**。S01–S04使用现有真实实例，没有新建Screen主组件或修改全局资产。

## S04验收摘要

九步主流程、手动补录、复盘结果、结构化事件、能力候选与最近复盘已组合。查看步骤与执行权限分离；第4步阻断时回看第2步或查看第8步，仍保持第4步Blocked。文件指纹变化使旧人工确认失效；写入中禁止重复提交；复盘生成不自动开放结算，结算不自动接受能力候选。

Review Hub `509:48662` 可达33状态和18弹窗。最终531条NAVIGATE/OVERLAY、54项CLOSE已核对；93项其他组件/继承动作单列。新增6条S01/S02/S03入站另计，全部进入 `505:34022` 未选择比赛，不伪装为已传递当前比赛、资料包、主题或权限。

深层检查13,298个未隐藏节点、5,246个嵌套实例、3,189个普通文字层。非预期越界、失效引用、未绑定solid颜色、顶层重叠、异常按钮高度均0。256个Disabled控件无反应连接；普通文字最低对比度Light **4.53:1**、Dark **6.86:1**；10处正常纵向滚动单列。

紧凑九步隐藏辅助说明但不缩小字号，结果指标2×2。22名球员手动录入名单独立滚动，提交栏不随名单滚走。局部修复低对比文字、Current标签歧义和18个空Dialog事实容器，没有修改全局组件。

## 历史与实现边界

S01原验收15状态；S02原验收33状态/7弹窗；S03原验收33状态/3弹窗。S01–S04累计114个状态画面，仍只计4条路由。历史记录中的未接线表述与旧计数属于当次快照，最新集成增量以总状态和当前页独立记录为准。

S03继续保持输入门禁、独立ModelProvider可用与真实结果返回分开；正常未接入入口不跳到条件成功画面。S02保持双方各11名首发、保存与模型准入独立及失败保留草稿等契约。

完整17路由集成仍未完成。所有输入、文件、任务和结算结果均为预置示例。S04六个写操作最终确认没有连接到伪造成功；文件按钮只展示预检说明。手动补录的合法任意提交、球员补充量化字段、全部选择器、刷新及完整状态矩阵不作为已实现内容。

真实文件/SHA、数据库事务、后台权限、结算与能力回写、键盘焦点、异步竞态及全状态主题/草稿/滚动持续性，仍需后续实现和集成测试。

## 换会话恢复

先读本README、`FIGMA_SCREENS_PLAN.md`、`FIGMA_SCREENS_STATE.json`，再读 `FIGMA_SCREEN_REVIEW_STATE.json` 和本页记录，核对实际Figma根节点。S04检查点已从分配阶段、组合阶段推进为accepted；不得再把早期WIP当成当前状态。

S03旧总状态保留于 `ff60437ccd38e1ed169cbebad0b21f7d8610e128`；S02旧总状态保留于 `6aaff66b66fdcdc9f93f2fb74436b9534412f5cc`；S01旧总状态保留于 `7a6891c571ffa07015774ac6b572c93b12c4c315`。各页独立记录继续保留。

`FIGMA_COMPONENT_RECORD.md`组件API保持不变；不要为了同步阶段状态重写未变化的组件历史。旧截图、空handoff或组件阶段Screens未开始文字不能覆盖最新明确检查点；SLOT虚拟ID按稳定根、组件引用及语义路径恢复，不猜ID。
