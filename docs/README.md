# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并成一份大记录。

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的 Figma 为准。应用源码只用于核对功能入口、交互和数据语义，不再作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

## 当前恢复入口

**2026-09-30 最新检查点：产品 Screens 已开始，S01 Dashboard / 数据总览的视觉与本页状态已验收。** 当前为 **1 / 17 条产品路由**，其余16条尚未开始。S01的15个状态画面不是15个不同页面；跨页面原型集成仍待目标页面完成。

- [Screens 总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新 Screens 连续性检查点与节点记录](FIGMA_SCREENS_STATE.json)
- [S01 Dashboard：页面、状态、验收与待接线目标](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S02 `lineups` / 比赛中心 · 比赛与阵容，尚未开始。**

## 组件与模式记录

- [组件阶段记录与 Figma 节点](FIGMA_COMPONENT_RECORD.md)
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
- [Workflow / Timeline 历史检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)
- [AI Chat：历史、消息、只读附件与 Composer](FIGMA_AI_CHAT.md)
- [AI Chat 历史检查点与节点账本](FIGMA_AI_CHAT_STATE.json)
- [应用图标待办](FIGMA_ICON_BACKLOG.md)
- [基础 Token 与底层组件待办](FIGMA_FOUNDATION_COMPONENT_BACKLOG.md)
- [Patterns 与 Screens 总阶段计划](FIGMA_PATTERN_SCREEN_PLAN.md)

执行顺序：图标 → 基础依赖与组件 → Patterns → Screens。

图标、既定底层组件及Patterns六组均已完成。Controls保持 **61个组件集 /506个变体**；Patterns保持 **17个组件集 /125个变体 /13个生产单组件**；Foundations保持 **193 variables /13 text styles /6 effects**。S01全部用现有实例组合，没有新增主组件或修改这些全局资产。

## S01 验收摘要

15个产品状态画面覆盖默认Light/Dark、桌面与紧凑侧栏展开/折叠、系统详情、数据库连接引导与失败/成功、首次加载、保留数据刷新、读取失败和已验证空数据。Review Hub仅用于验收目录，不计作产品路由。

深层检查3,699个可见节点、1,422个嵌套实例，非预期越界、失效引用、未绑定可见颜色、顶层重叠均为0。47条本页与目录导航连接已核对；137条继承组件交互单独统计。134个可见Disabled控件均无反应连接。普通文字最低对比度Light 4.53:1 /Dark 6.86:1。

连接错误页正常的纵向滚动单独记录，不以裁切隐藏问题。五个快捷入口的目标路由已记录，但真实跨页面连接尚未完成，不跳转到旧Pattern QA假装产品完成。

本轮没有修改应用源码或既有组件主节点。所有数据为演示值，连接/读取结果为预置模拟；不替代真实API、数据库连接、输入校验、键盘焦点、权限、异步与持久化测试。

## 换会话恢复

先读本README和 `FIGMA_SCREENS_PLAN.md`、`FIGMA_SCREENS_STATE.json`，再核对当前屏幕与实际Figma节点。旧截图、AI Chat/Workflow状态文件及组件记录中的“Screens未开始”属于先前阶段快照，不能覆盖本次最新检查点。

`FIGMA_COMPONENT_RECORD.md` 的组件API与历史记录本轮保持不变；Screens当前进度以专门计划和状态文件为准。不要为了同步一个阶段状态而重写整份未变化的组件历史。
