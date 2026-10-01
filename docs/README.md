# Figma Design System Docs

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支 `ui-design-system`。

## 当前恢复入口

**2026-10-01最新检查点：S10 rules / 规则与模型的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **10 / 17条产品路由**，S11–S17七条尚未开始。完整运行时和跨页面端到端集成未完成。

- [Screens总计划](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S10规则与模型：层级、路由、注册及源码差异](FIGMA_SCREEN_RULES.md)
- [S10完整节点与恢复状态](FIGMA_SCREEN_RULES_STATE.json)
- [S09 Excel工作包](FIGMA_SCREEN_WORKBOOKS.md) / [独立状态](FIGMA_SCREEN_WORKBOOKS_STATE.json)
- [S08阵容预设](FIGMA_SCREEN_LINEUP_PRESETS.md) / [独立状态](FIGMA_SCREEN_LINEUP_PRESETS_STATE.json)
- [S07球员](FIGMA_SCREEN_PLAYERS.md) / [独立状态](FIGMA_SCREEN_PLAYERS_STATE.json)
- [S06球队](FIGMA_SCREEN_TEAMS.md) / [独立状态](FIGMA_SCREEN_TEAMS_STATE.json)
- [S05运行记录](FIGMA_SCREEN_RUNS.md) / [独立状态](FIGMA_SCREEN_RUNS_STATE.json)
- [S04赛后复盘](FIGMA_SCREEN_REVIEW.md) / [独立状态](FIGMA_SCREEN_REVIEW_STATE.json)
- [S03赛事推演](FIGMA_SCREEN_PREDICTION.md) / [独立状态](FIGMA_SCREEN_PREDICTION_STATE.json)
- [S02比赛与阵容](FIGMA_SCREEN_LINEUPS.md) / [独立状态](FIGMA_SCREEN_LINEUPS_STATE.json)
- [S01数据总览](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S11 `release` / 发布验收，尚未开始。** S10有35个产品状态、27个弹窗和1个Review Hub；仍只计一条路由。S01–S10累计315个状态，不是315条路由。

## S10本轮验收

赛事目录、赛事结构、模型路由和规则包四区完成。层级切换后清除不匹配父ID；目录级路径不等于本场执行路由。删除赛事实际为停用目录和赛事级绑定，保留比赛、推演和复盘历史。

规则包读取、后端校验、注册、绑定和执行资格独立。历史P7只读；无效/变化文件、读取中、注册中、结果未确认和已注册读回均禁用当前注册操作。11个最终确认均无执行连接，未做真实创建、停用、绑定或注册。

Review Hub `591:124271` 可达35状态与27弹窗。最终 **697条NAVIGATE/OVERLAY、81项CLOSE、0其他动作**，外部6条入站另计：S01/S03/S09默认Light/Dark的模型导航进入真实S10。只保留代表性主题，不移交比赛、文件或授权。

深层8,939个未隐藏节点、3,559个真实嵌套实例；非预期越界、失效引用/目标、未绑定可见solid颜色、顶层重叠和异常按钮高度均0。205个禁用/忙控件无反应，父级点击旁路0；23处正常纵向滚动。普通文字最低对比度Light4.5327、Dark6.8625。

发现并记录四项源码差异：绑定表单可选赛事但处理器要求赛事；目录与路径查询P4过滤不一致；倒序列表按key建Map可能保留旧版本；规则加载浅校验与候选失效保护需运行时完善。**设计验收不代表这些源码问题已修复，也不是后端全面审计。**

## 组件与模式记录

- [组件阶段总记录](FIGMA_COMPONENT_RECORD.md)
- [按钮、Spinner与Loading](FIGMA_BUTTON_COMPONENTS.md)
- [扩展字段](FIGMA_EXTENDED_FIELDS.md)
- [Searchable Combobox](FIGMA_SEARCHABLE_COMBOBOX.md)
- [Switch / Toggle](FIGMA_SWITCH_TOGGLE.md)
- [Tabs / Segmented Control](FIGMA_TABS_SEGMENTED_CONTROL.md)
- [Data Table / Pagination](FIGMA_DATA_TABLE_PAGINATION.md)
- [Dropdown / Context / Overflow Menu](FIGMA_DROPDOWN_CONTEXT_OVERFLOW_MENU.md)
- [Dialog](FIGMA_DIALOG.md)
- [Toast / Inline Alert / Blocking Message / Task Activity](FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md)
- [Accordion / Disclosure](FIGMA_ACCORDION_DISCLOSURE.md)
- [Progress / Skeleton / Empty State](FIGMA_PROGRESS_SKELETON_EMPTY_STATE.md)
- [Avatar](FIGMA_AVATAR.md)
- [PC App Shell](FIGMA_PC_APP_SHELL.md)
- [Page Heading / Toolbar / Filter / Selection](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)
- [Metric Card / Action Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)
- [Master–Detail–Inspector / Entity Row](FIGMA_MASTER_DETAIL_INSPECTOR.md)
- [Workflow / Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md) / [历史检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)
- [AI Chat](FIGMA_AI_CHAT.md) / [历史检查点](FIGMA_AI_CHAT_STATE.json)
- [图标待办](FIGMA_ICON_BACKLOG.md)
- [底层Token与组件待办](FIGMA_FOUNDATION_COMPONENT_BACKLOG.md)
- [Patterns与Screens总阶段计划](FIGMA_PATTERN_SCREEN_PLAN.md)

底层组件与六组Patterns全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S10复用真实实例，没有创建Screen主组件、detach、修改共享资产或应用源码。

## 历史与运行时边界

历史各页计数、节点、验收及未实现事项以各自独立状态为准。S10没有重写为全产品通过：S02双方11首发与准入独立；S03输入、提供器和输出独立；S04查看/执行与复盘/结算分开；S05隐藏不删血缘；S06归档/删除/强清和导入/P4资格分开；S07来源身份、可用性、能力和短期标签独立；S08预设与比赛副本分离；S09类别/文件/模式/批次隔离，结果不明不重发。AI Chat保留只读上下文与session/request隔离。

S10任意筛选/输入、所有对象与包版本、真实文件/schema/兼容性验证、CRUD/停用/绑定/注册事务与幂等、完整创建/失败控制器和所有主题/折叠/滚动/草稿/候选/焦点持续性尚未实现。局部导航或禁用画面不证明全部跨页竞态正确。所有数据、文件和结果均为预置演示。

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_RULES_STATE.json`及S10记录 → 实际稳定根。虚拟SLOT节点按根、语义名称和真实组件引用解析，不猜ID。

S09总状态保留于 `4627fecbe7d83ce377c36db82d63aec2bed34712`；S08于 `06eee1a492a92b620485d194cdd83844ecadef13`；更早版本见各阶段恢复链。初始/组合WIP已被accepted覆盖。旧截图、空handoff、旧“Screens未开始”记录不能替代最新明确检查点；未修改组件API不为更新进度重写。
