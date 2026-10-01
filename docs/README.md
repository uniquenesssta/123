# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并为一份大记录。

**设计依据（2026-08-28用户确认）：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义，不提供新视觉几何。**

## 当前恢复入口

**2026-10-01最新检查点：S09 workbooks / Excel工作包的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **9 / 17条产品路由**：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks。S10–S17共8条尚未开始。

- [Screens总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S09工作包：类别、批次、预检和提交边界](FIGMA_SCREEN_WORKBOOKS.md)
- [S09全部节点与明确恢复状态](FIGMA_SCREEN_WORKBOOKS_STATE.json)
- [S08阵容预设：原验收记录](FIGMA_SCREEN_LINEUP_PRESETS.md)
- [S08独立恢复状态](FIGMA_SCREEN_LINEUP_PRESETS_STATE.json)
- [S07球员](FIGMA_SCREEN_PLAYERS.md)
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

**下一项：S10 `rules` / 规则与模型，尚未开始。** S09有30个产品状态、26个弹窗和1Review Hub，仍只计一条路由。S01–S09累计280个状态，不是280条路由；此前页内工作包原型不代替这次独立S09验收。

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

执行顺序：图标 → 基础依赖与组件 → Patterns → Screens。底层组件和Patterns六组全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S09没有创建Screen主组件、detach或修改共享资产。

## S09验收摘要

球队月度、球员月度、比赛与阵容分别拥有文件、模式、批次和预检；模板导出、现有数据导出、预检、确认提交不是同一个操作。无变更、忙状态、已导入/重复、文件/模式变化和结果未确认分别禁提交；空白默认保留，clear显式清空但仍须校验。

球队、球员和比赛使用T-DEMO-01、P-DEMO-01、M-DEMO-01；仅新增重新预检使用T-DEMO-02。Team Ready → Player Isolated → Team Ready保留球队预检的限定路径可用；其他多批次组合转只读说明，不假装全部持续性已实现。已有/不明批次不会借演示文件选择器返回旧Ready绕过保护。

Review Hub `582:112939` 可达30状态、26弹窗。最后读回 **585条NAVIGATE/OVERLAY、78项CLOSE、0其他动作**；7条外部入站另计。6条S06/S07/S08目录Light/Dark工作包入口保留代表性主题；S07“全部工作包”只保留球员类别，不转移文件/批次。

深层8,190个未隐藏节点、3,186个真实嵌套实例、2,234个普通文字层；非预期越界、失效引用/目标、未绑定可见solid颜色、顶层重叠及异常按钮高度均0。167个禁用控件及父级点击旁路检查通过；17处正常滚动，提交栏在外。最低普通文字对比度Light **4.5327:1**、Dark **6.8625:1**。

10个最终操作确认（6导出、4导入）反应全部0，没有执行文件或数据库写入。批次明细是结构说明，不是真实完整JSON；成功/错误由验收目录独立选择。

**源码实现待办：** workbooks.ts的ready表达式只检查preview及冲突/错误；本页新增的无变更、忙状态、已提交、失效上下文和结果不明保护仍需核对并落实到运行时。本轮未修改源码，也不声称其他后端层没有任何保护。

## 历史与实现边界

S01–S08分别为15、33、33、33、18、42、45、31个状态。原验收计数和未实现内容保留于独立状态；后续入口以最新总检查点为准，不将有限原型回写为全产品集成通过。

S02双方11首发、保存与准入独立；S03输入/ModelProvider/输出独立；S04查看/执行分离、SHA变化拒绝旧确认、复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强制清除及导入/P4资格分开；S07有效履历、可用性、能力与短期标签分开；S08同队、11首发、唯一槽位及只替换目标侧草稿的规则保留。AI Chat维持只读上下文和session/request隔离。

S08原31状态/20弹窗、516条导航/60关闭、7条入站、11个最终确认无执行，原完整编辑以A-P1为代表；S09没有修改其数据或验证规则，只接两条通用工作包入口。

S09任意文件/模式输入、完整多批次控制器、冲突处理、导出范围与覆盖、真实解析/预检/提交/幂等/核验，以及全状态主题/滚动/焦点/上下文持续性尚未实现。完整17路由接线、真实权限/竞态、模型/Worker及此前遗留运行时缺口仍需后续完成。

## 换会话恢复

先读本README、`FIGMA_SCREENS_PLAN.md`、`FIGMA_SCREENS_STATE.json`，再读 `FIGMA_SCREEN_WORKBOOKS_STATE.json`与S09记录，核对实际稳定根。S09初始和组合WIP由accepted替代，不重复创建。

S08旧总检查点保留于 `06eee1a492a92b620485d194cdd83844ecadef13`；S07于 `66d575f0d9985012a43448e2fed2af9f913c3117`；S06于 `ac85c1b42be55bd7378b0990a3406b6da4d9d087`。更早版本见Screens计划及Git历史。

不要为更新进度重写未变更组件API。旧截图、空handoff、组件阶段“Screens未开始”不能覆盖最新明确状态；规范化SLOT虚拟ID按稳定根、真实组件和语义路径恢复，不猜ID。Create State本次无新handoff，GitHub明确状态是恢复依据。
