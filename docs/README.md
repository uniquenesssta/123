# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并成一份大记录。

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的 Figma 为准。应用源码只用于核对功能入口、交互和数据语义，不再作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

## 当前恢复入口

**2026-09-30 最新检查点：S02 lineups / 比赛中心·比赛与阵容的视觉、本页适用状态和有限原型已验收。** Screens当前为 **2 / 17 条产品路由**：S01 dashboard、S02 lineups；其余15条尚未开始。S02的33个状态画面、7个对话框不等于40条产品页面。

- [Screens总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S02比赛与阵容：设计、状态、原型及验收边界](FIGMA_SCREEN_LINEUPS.md)
- [S02明确恢复状态与节点账本](FIGMA_SCREEN_LINEUPS_STATE.json)
- [S01 Dashboard：页面与原验收记录](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S03 `prediction` / 赛事推演，尚未开始。**

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

图标、既定底层组件及Patterns六组均已完成。Controls保持 **61 sets /506 variants**；Patterns保持 **17 sets /125 variants /13 production singles**；Foundations保持 **193 variables /13 text styles /6 effects**。S01和S02使用现有实例组合，没有新建Screen主组件或修改全局资产。

## S02验收摘要

五个页内区域为比赛管理、双方阵容、模型链路、版本历史、Excel工作包。双方首发11+11方可提交；缺少首发、重复提交、导入冲突与格式错误保持禁用。保存失败保留草稿，保存成功不自动解除模型链路阻断。高级设置、预设和危险确认的运行限制见独立记录。

33个产品状态和7个对话框可从Review Hub `478:22468` 到达。最终339条NAVIGATE/OVERLAY连接、23项CLOSE动作已回读；399个Disabled控件没有反应连接。5,544个未隐藏嵌套实例可解析；非预期越界、失效引用、未绑定solid颜色和顶层重叠为0。19处正常纵向滚动单独记录，阵容提交栏位于名单滚动区之外。

2,957个普通非Disabled文字层最低对比度Light **4.5048:1** / Dark **5.6558:1**。131个局部文字颜色覆盖复用既有primary token，没有修改全局组件；截图发现的多行链路说明裁切也已修正。

S01默认Light/Dark的比赛快捷卡与“比赛”主导航已有4条连接指向真实S02，S02有明确返回首页入口。克隆带入的60条错误旧首页主题/折叠连接已移除。全产品跨路由、全状态主题/折叠/草稿/滚动持续性仍未完成；其余产品目标不连接旧QA示例冒充完成。

本轮没有修改应用源码、全局变量或既有组件主节点。数据和请求结果均为预置演示；真实输入、API、数据库事务、文件操作、权限、键盘焦点与异步竞态须在实现阶段测试。最终删除/归档确认不执行真实或伪造的成功写操作。

## S01历史与当前补充

S01原验收为15个状态画面、47条本页/目录连接及134个无反应的Disabled控件。该独立记录中的“尚无跨页面连接”是S01验收时的历史状态；当前S01↔S02补充以最新Screens计划和S02记录为准。

## 换会话恢复

先读本README、`FIGMA_SCREENS_PLAN.md`、`FIGMA_SCREENS_STATE.json`，再读 `FIGMA_SCREEN_LINEUPS_STATE.json` 和对应独立记录，核对实际Figma节点。S01完整旧总状态保留在提交 `7a6891c571ffa07015774ac6b572c93b12c4c315`。

旧截图、AI Chat/Workflow状态和组件记录中的“Screens未开始”属于历史快照，不能覆盖最新检查点。`FIGMA_COMPONENT_RECORD.md`的组件API本轮保持不变；不要为了同步一个阶段进度而重写未变化的组件历史。
