# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并成一份大记录。

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的 Figma 为准。应用源码只用于核对功能入口、交互和数据语义，不再作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

- [组件当前状态与 Figma 节点](FIGMA_COMPONENT_RECORD.md)
- [按钮、Spinner 与 Loading 组件](FIGMA_BUTTON_COMPONENTS.md)
- [扩展字段组件](FIGMA_EXTENDED_FIELDS.md)
- [Searchable Combobox 组件](FIGMA_SEARCHABLE_COMBOBOX.md)
- [Switch / Toggle 组件](FIGMA_SWITCH_TOGGLE.md)
- [Tabs / Segmented Control 组件](FIGMA_TABS_SEGMENTED_CONTROL.md)
- [Data Table / Pagination 组件](FIGMA_DATA_TABLE_PAGINATION.md)
- [Dropdown / Context / Overflow Menu 组件](FIGMA_DROPDOWN_CONTEXT_OVERFLOW_MENU.md)
- [Dialog：普通、确认与名称校验危险确认](FIGMA_DIALOG.md)
- [Toast / Inline Alert / Blocking Message 与 Task Activity](FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md)
- [Accordion / Disclosure 与展开状态恢复](FIGMA_ACCORDION_DISCLOSURE.md)
- [Progress / Skeleton / Empty State 与异步状态边界](FIGMA_PROGRESS_SKELETON_EMPTY_STATE.md)
- [Avatar：球队、球员与默认占位](FIGMA_AVATAR.md)
- [PC App Shell：导航、顶栏与交互验收](FIGMA_PC_APP_SHELL.md)
- [Page Heading / Toolbar / Filter Bar / Selection Command Bar](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)
- [Metric Card / Action Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)
- [Master–Detail–Inspector 与 Entity Row](FIGMA_MASTER_DETAIL_INSPECTOR.md)
- [Workflow Stepper / Timeline 与交互验收](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)
- [Workflow / Timeline 连续性检查点与节点账本](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)
- [AI Chat：历史、消息、只读附件与 Composer](FIGMA_AI_CHAT.md)
- [最新 AI Chat 连续性检查点与节点账本](FIGMA_AI_CHAT_STATE.json)
- [应用图标待办](FIGMA_ICON_BACKLOG.md)
- [基础 Token 与底层组件待办](FIGMA_FOUNDATION_COMPONENT_BACKLOG.md)
- [Patterns 与 Screens 计划](FIGMA_PATTERN_SCREEN_PLAN.md)

执行顺序：图标 → 基础依赖与组件 → Patterns → Screens。

当前进度：图标、基础依赖、按钮首组、扩展字段组、Searchable Combobox、Switch / Toggle、Tabs / Segmented Control、Data Table / Pagination、Dropdown / Context / Overflow Menu、Dialog、Toast / Inline Alert / Blocking Message（含 Task Activity）、Accordion / Disclosure、Progress / Skeleton / Empty State 及 Avatar 均已完成并验收。Controls 当前为 **61 个组件集、506 个变体**；既定底层组件剩余 **0 组**。

**2026-09-30 最新检查点：AI Chat 已完成，Patterns 六组全部完成。** 六组为 PC App Shell、Page Heading / Toolbar / Filter Bar / Selection Command Bar、Metric Card / Action Card / Panel、Master–Detail–Inspector、Workflow Stepper / Timeline、AI Chat。

Patterns 当前为 **17 个组件集 / 125 个变体 / 13 个生产单组件**；Foundations 保持 **193 variables / 13 text styles / 6 effects**。AI Chat 新增4个组件集、21个变体及2个单组件，没有新增全局变量或样式。

本组覆盖 Light / Dark、760px 窄宽、完整组件状态、只读上下文主动勾选、历史无匹配、长普通文本和独立滚动。7个预置原型场景、17条连接均已读回；A/B 会话隔离、取消/失败保留草稿均有明确示例。深层检查351个可见嵌套实例，非预期越界与失效引用为0；普通文字最低对比度 Light 4.53:1 / Dark 6.86:1。

**下一阶段：产品 Screens（17条页面链路 / 7个模块），尚未开始。** Review、AI Chat Lab 和可复用 Workspace 都不是已完成的产品页面。本轮没有修改应用源码或既有 Controls 主组件。原型是有限预置模拟，不替代真实 API、键盘焦点、权限、异步与持久化测试。

换会话恢复：先读当前计划与 `FIGMA_AI_CHAT_STATE.json`，再核对 Figma 节点；旧截图、旧组的历史总数和仅有标题的 handoff 不能覆盖更新后的明确检查点。
