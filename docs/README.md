# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并为一份大记录。

**设计依据（2026-08-28用户确认）：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义，不提供新视觉几何。**

## 当前恢复入口

**2026-10-01最新检查点：S07 players / 球员的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **7 / 17条产品路由**：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players。S08–S17共10条尚未开始。

- [Screens总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S07球员：目录、来源身份、履历、观察和工作包](FIGMA_SCREEN_PLAYERS.md)
- [S07全部节点与明确恢复状态](FIGMA_SCREEN_PLAYERS_STATE.json)
- [S06球队：原验收记录](FIGMA_SCREEN_TEAMS.md)
- [S06独立恢复状态](FIGMA_SCREEN_TEAMS_STATE.json)
- [S05运行记录](FIGMA_SCREEN_RUNS.md)
- [S05独立恢复状态](FIGMA_SCREEN_RUNS_STATE.json)
- [S04赛后复盘](FIGMA_SCREEN_REVIEW.md)
- [S04独立恢复状态](FIGMA_SCREEN_REVIEW_STATE.json)
- [S03赛事推演](FIGMA_SCREEN_PREDICTION.md)
- [S03独立恢复状态](FIGMA_SCREEN_PREDICTION_STATE.json)
- [S02比赛与阵容](FIGMA_SCREEN_LINEUPS.md)
- [S02独立恢复状态](FIGMA_SCREEN_LINEUPS_STATE.json)
- [S01数据总览](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S08 `lineup_presets` / 阵容预设，尚未开始。** S07有45个产品状态、30弹窗和1Review Hub，仍只计一条路由。S01–S07累计219个状态，不是219条路由；球员工作包不代表S09通用工作簿完成。

## 组件与模式记录

- [组件阶段记录与Figma节点](FIGMA_COMPONENT_RECORD.md)
- [按钮、Spinner与Loading](FIGMA_BUTTON_COMPONENTS.md)
- [扩展字段](FIGMA_EXTENDED_FIELDS.md)
- [Searchable Combobox](FIGMA_SEARCHABLE_COMBOBOX.md)
- [Switch / Toggle](FIGMA_SWITCH_TOGGLE.md)
- [Tabs / Segmented Control](FIGMA_TABS_SEGMENTED_CONTROL.md)
- [Data Table / Pagination](FIGMA_DATA_TABLE_PAGINATION.md)
- [Dropdown / Context / Overflow Menu](FIGMA_DROPDOWN_CONTEXT_OVERFLOW_MENU.md)
- [Dialog与危险确认](FIGMA_DIALOG.md)
- [Toast / Inline Alert / Blocking Message与Task Activity](FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md)
- [Accordion / Disclosure](FIGMA_ACCORDION_DISCLOSURE.md)
- [Progress / Skeleton / Empty State](FIGMA_PROGRESS_SKELETON_EMPTY_STATE.md)
- [Avatar](FIGMA_AVATAR.md)
- [PC App Shell](FIGMA_PC_APP_SHELL.md)
- [Page Heading / Toolbar / Filter Bar / Selection Command Bar](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)
- [Metric Card / Action Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)
- [Master–Detail–Inspector与Entity Row](FIGMA_MASTER_DETAIL_INSPECTOR.md)
- [Workflow Stepper / Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)
- [Workflow / Timeline历史检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)
- [AI Chat](FIGMA_AI_CHAT.md)
- [AI Chat历史检查点](FIGMA_AI_CHAT_STATE.json)
- [应用图标待办](FIGMA_ICON_BACKLOG.md)
- [基础Token与底层组件待办](FIGMA_FOUNDATION_COMPONENT_BACKLOG.md)
- [Patterns与Screens总阶段计划](FIGMA_PATTERN_SCREEN_PLAN.md)

执行顺序：图标 → 基础依赖与组件 → Patterns → Screens。

底层组件及Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。S07继续复用真实实例，没有创建Screen主组件、detach或修改共享资产。

## S07验收摘要

目录、筛选、分页、来源球队列表、批量选择示例、A01/A02/D01速览、A01完整档案、名称、位置、履历、可用性、能力观察、动态标签、外部编号、球员工作包、新增及连接前置已组合。其他球员保留各自只读摘要，不套用A01字段。

独立目录不沿用旧来源球队；从S06 A01速览进入 `550:73700`，两个返回按钮均到首尔FC球队档案 `527:57101`。本次新增五条S06入口，另更新一处待接线说明；不是所有比赛/球队来源与表单持续性已完成。

Review Hub **`557:87148`** 可达45状态和30弹窗。最终 **925条NAVIGATE/OVERLAY、90项CLOSE、0其他动作**；五条外部入站另计。12个最终写确认均0反应，成功/失败场景由评审目录独立选择。

深层 **13,520个未隐藏节点、5,575个嵌套实例**均检查。非预期越界、失效引用、未绑定可见颜色、异常按钮高度、顶层重叠和失效原型目标均0。265禁用/忙控件无反应，可点击父容器旁路0；13处正常垂直滚动单列。3,493个普通文字层最低对比度Light **4.5048:1**、Dark **5.5998:1**。

截图驱动修正MDI原示例裁切边界，操作列与分页已完整可见。另修复表格文字、换行网格、紧凑导航、低对比文字、窄Skeleton和未知国籍空值；共享主组件未改。

## 语义与实现边界

球员生命周期与比赛可用性分开；当前球队按有效履历判断；短期标签不覆盖永久能力。未知年龄/能力不补0或默认50，中文姓名留空保留现有值。能力101/可信度1.20/样本-1的示例禁止保存。

固定时点下标签分别为有效、已过期、未生效；贡献按钮只展示范围说明，没有真实算法。源表单编辑身高120–230、新增100–240的差异保留为后续统一校验待办，不宣称源码已修正。

导入仅接受球员文件；冲突、错误、无待处理项、写入中和结果不明均受控。归档保留历史，永久删除只针对无引用C01，保护A01；没有添加球员强制删除入口。新增不自动加入来源球队或创建默认能力。

所有球员、球队、日期、评分、批次和结果都是预置示例。任意输入、真实多选/筛选/游标请求、全部球员编辑、文件读写、数据库事务、后端权限、键盘焦点、异步竞态，以及全状态主题/来源/草稿/滚动持续性仍未实现。部分导出、AI、未来路由和表单控件仅有视觉与语义，不声明全部可交互。

S01–S06原验收与约束继续保留：S02双方11首发、保存与准入分离；S03输入/提供器/输出独立；S04查看/执行/结算分离与不可变证据；S05隐藏不删血缘；S06归档/普通删除/强制清除及资料导入/P4资格分开。各页独立记录中的旧计数属于当次快照，后续集成以最新总状态为准。

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_PLAYERS_STATE.json`与S07记录 → Figma实际稳定根及SLOT/实例引用。

S06旧总检查点 `ac85c1b42be55bd7378b0990a3406b6da4d9d087`；S05 `444ad13d6e2de9fe9d99eb9a3c15022b40cd2ef4`；S04 `1d2b9b78d4d7bf2e589c81e69e4fef89638c98cb`；S03 `ff60437ccd38e1ed169cbebad0b21f7d8610e128`；S02 `6aaff66b66fdcdc9f93f2fb74436b9534412f5cc`；S01 `7a6891c571ffa07015774ac6b572c93b12c4c315`。

S07分配/组合WIP已被accepted替代。不要根据旧截图、旧WIP、空handoff或组件阶段Screens未开始文字回退。Create State本轮未新建handoff，GitHub明确状态是权威恢复入口；不得把未尝试写入算作成功。未变化的组件API不重写；规范化SLOT虚拟ID按根、语义路径和实际引用恢复，不猜ID。
