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
- [应用图标待办](FIGMA_ICON_BACKLOG.md)
- [基础 Token 与底层组件待办](FIGMA_FOUNDATION_COMPONENT_BACKLOG.md)
- [Patterns 与 Screens 计划](FIGMA_PATTERN_SCREEN_PLAN.md)

执行顺序：图标 → 基础依赖与组件 → Patterns → Screens。

当前进度：图标、基础依赖、按钮首组、扩展字段组、Searchable Combobox、Switch / Toggle、Tabs / Segmented Control、Data Table / Pagination、Dropdown / Context / Overflow Menu、Dialog、Toast / Inline Alert / Blocking Message（含 Task Activity）、Accordion / Disclosure、Progress / Skeleton / Empty State 及 Avatar 均已完成并验收。Controls 当前为 **61 个组件集、506 个变体**；既定底层组件剩余 **0 组**。

Patterns 首组 **PC App Shell** 已完成：9 个组件集、60 个变体、2 个生产单组件，另有 1 个 QA 夹具；7 个模块 / 17 个入口的双主题、展开与折叠原型已连接。下一组为 **Page Heading / Toolbar / Filter Bar / Selection Command Bar**。Screens 仍为空；本轮没有修改应用源码。
