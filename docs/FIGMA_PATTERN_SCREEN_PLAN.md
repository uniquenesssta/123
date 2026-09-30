# Figma Patterns & Screen Plan

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的Figma为准。应用源码只用于核对功能入口、交互和数据语义，不作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

## 当前状态

截至 **2026-09-30**，图标与既定底层组件均完成，Controls保持 **61 sets /506 variants**。**Patterns六组全部完成**：PC App Shell、Page Heading / Toolbar / Filter Bar / Selection Command Bar、Metric Card / Action Card / Panel、Master–Detail–Inspector、Workflow Stepper / Timeline、AI Chat。

**Screens已有2 / 17条产品路由完成视觉与本页适用状态验收：S01 dashboard、S02 lineups。其余15条尚未开始。** S01的15个状态、S02的33个状态仍分别只计一条路由；对话框、Review Hub和Pattern QA不增加页面路由计数。

最新恢复入口：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S02详细状态](FIGMA_SCREEN_LINEUPS_STATE.json) → [S02独立记录](FIGMA_SCREEN_LINEUPS.md)。执行顺序见[17路由Screens计划](FIGMA_SCREENS_PLAN.md)。**下一项：S03 `prediction` / 赛事推演，本轮未开始。**

Patterns保持 **17 sets /125 variants /13 production singles**，Foundations保持 **193 variables /13 text styles /6 effects**。S02只使用现有真实实例；没有新建Screen主组件、修改应用源码或全局组件/变量。

## Patterns执行记录

- [x] PC App Shell：一级导航、二级导航、Topbar、折叠与主题保持。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector工作区（含Entity Row依赖）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件、Composer。

依赖按组归档，不合并为一份大文档。证据见[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)、[Master–Detail–Inspector](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow / Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat检查点](FIGMA_AI_CHAT_STATE.json)与[Workflow检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)保留为历史记录；其中Screens未开始字段已由更新的Screens检查点取代。

## 当前阶段：Screens

- [x] S01 `dashboard`：数据总览视觉与本页状态。
- [x] S02 `lineups`：比赛管理、双方阵容、模型链路、阵容历史、页内工作包。
- [x] S01默认Light/Dark的比赛快捷卡与主导航回填到真实S02；S02返回首页。
- [ ] **S03 `prediction`：下一项，尚未开始。**
- [ ] S04–S17：均尚未开始；S02页内工作包不代表S09独立workbooks页面完成。
- [ ] 其余真实路由目标回填、全状态主题/折叠持续性和完整跨路由联调。

S02有33个产品状态、7个对话框、1个Review Hub。最终339条NAVIGATE/OVERLAY连接和23项CLOSE动作已回读；S01到S02的4条新增入站连接另计。399个Disabled控件无反应连接。5,544个未隐藏嵌套实例无失效引用；非预期越界、未绑定solid颜色、顶层重叠为0；19处正常纵向滚动单列。普通文字最低对比度Light 4.5048:1 / Dark 5.6558:1。

克隆带入的60条旧S01主题/折叠误跳转已清理，不能用错误目标代替未实现交互。最新结果与具体限制见S02独立记录。S01的历史验收证据继续保留于[S01记录](FIGMA_SCREEN_DASHBOARD.md)。

## 页面与实现边界

产品页面只引用已完成组件和模式；必要新依赖单独补缺。控件来源、实际接线与仅记录的语义目标须可区分；未完成目标不得通过旧QA插槽冒充完成。

本页设计、有限原型、跨路由集成、真实应用测试为不同验收层。数据、保存、导入与请求结果均为预置示例，不执行真实数据库、API或文件操作。最终删除/归档写操作未接线，高级设置/预设确认仅关闭示例覆盖层；不能宣称所有控件都已实现。

S02保持双方各11名首发才能提交、失败保留草稿、保存与模型准入独立、导入先预检后明确确认、错误不自动重发写操作。实际键盘焦点、竞态、事务、权限、跨页面表单/滚动/主题持续性需后续运行时测试。

保持其他既有边界：Workflow查看步骤与执行权限独立；Timeline核验、修订、取消及比分由真实数据驱动；AI Chat纯文本、只读上下文主动勾选、Pending/取消绑定sessionId/requestId；附件不扩展为上传、生成文件或数据库执行。

恢复时先读本计划与最新Screens状态，再核对当前画布。组件总记录中的Screens未开始文字和旧阶段统计是历史快照，本轮不重写未变化的组件API。
