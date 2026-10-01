# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并为一份大记录。

**设计依据（2026-08-28用户确认）：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义，不提供新视觉几何。**

## 当前恢复入口

**2026-10-01最新检查点：S08 lineup_presets / 阵容预设的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **8 / 17条产品路由**：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets。S09–S17共9条尚未开始。

- [Screens总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S08阵容预设：编辑校验、身份、操作与套用边界](FIGMA_SCREEN_LINEUP_PRESETS.md)
- [S08全部节点与明确恢复状态](FIGMA_SCREEN_LINEUP_PRESETS_STATE.json)
- [S07球员：原验收记录](FIGMA_SCREEN_PLAYERS.md)
- [S07独立恢复状态](FIGMA_SCREEN_PLAYERS_STATE.json)
- [S06球队](FIGMA_SCREEN_TEAMS.md)
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

**下一项：S09 `workbooks` / Excel工作包，尚未开始。** S08有31个产品状态、20弹窗和1Review Hub，仍只计一条路由。S01–S08累计250个状态，不是250条路由；其他页面的局部工作包不代表S09完成。

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

执行顺序：图标 → 基础依赖与组件 → Patterns → Screens。既有底层组件和Patterns六组全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。本轮继续使用真实实例，没有创建Screen主组件、detach或修改共享资产。

## S08验收摘要

球队目录、活动/归档列表、默认方案、管理入口、代表性完整编辑、空/错误/刷新、保存校验、复制/归档/永久删除确认、比赛套用规则和数据库前置完成。

首发必须恰好11名，战术位置唯一且完整匹配阵型槽位；使用概率可空或0–1。保存中和结果不明禁止重复写入；v4成功读回不再提交旧v3确认。A-P1为完整编辑代表，其他球队/预设保持自己的只读说明，不跳入A字段。

管理入口可永久删除活动或归档预设，但范围仅预设及其成员，不删除球队、球员或比赛副本。套用另做新预检，确认can_apply和同队身份，只替换目标侧草稿。11个最终操作确认均无执行连接。

Review Hub `573:103157` 可达31状态与20弹窗。最终 **516条NAVIGATE/OVERLAY、60项CLOSE、0其他动作**；外部入站7条单独计数。S06/S07通用入口不伪造已选球队，S02主/客管理入口分别保持首尔FC和全北现代身份。

深层12,307个未隐藏节点、5,139个真实嵌套实例；非预期越界、失效引用/目标、未绑定可见颜色、顶层重叠和异常按钮高度均0。249个禁用/忙控件无反应、父容器旁路0；30处正常滚动。普通文字最低对比度Light **4.5048:1**、Dark **4.5490:1**。

## 历史与实现边界

S01–S07分别为15、33、33、33、18、42、45个状态；原验收计数和未实现内容保留于各页独立状态。新增入口以最新总检查点为准，不将历史有限原型改写为全产品通过。

S02双方11首发、保存与准入独立；S03输入/ModelProvider/输出独立；S04查看/执行分离、SHA变化拒绝旧确认、复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强制清除及导入/P4资格分开；S07有效履历、可用性、能力与短期标签分开。AI Chat维持只读上下文和session/request隔离。

S08任意成员选择/字段输入/筛选、全部球队与预设编辑、自动分配与角色继承算法、实时伤停预检、真实保存/复制/归档/删除事务、草稿覆盖以及完整主题/滚动/焦点/上下文持续性尚未实现。复制、归档和删除全过程提交/错误控制器未计为完成。套用通过状态只是条件示例，不能当作已写入比赛。

完整17路由集成、真实权限/竞态、文件/数据库、模型/Worker及先前页面保留的运行时缺口均需后续完成。所有球队、成员、版本与结果是预置演示。

## 换会话恢复

先读本README、`FIGMA_SCREENS_PLAN.md`、`FIGMA_SCREENS_STATE.json`，再读 `FIGMA_SCREEN_LINEUP_PRESETS_STATE.json` 与S08记录，核对实际稳定根。S08初始和组合WIP已被accepted替代，不重复构建。

S07旧总状态保留于 `66d575f0d9985012a43448e2fed2af9f913c3117`，S06于 `ac85c1b42be55bd7378b0990a3406b6da4d9d087`；更早版本索引见Screens计划。一次工具连接错误后已先回读确认画布改动，再进行最终只读验收。

不要为更新进度重写未变化的组件API。旧截图、空handoff、组件阶段“Screens未开始”不能覆盖最新明确状态；规范化SLOT虚拟ID按稳定根、真实组件和语义路径恢复，不猜ID。
