# Figma Design System Docs

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支 `ui-design-system`。

## 当前恢复入口

**2026-10-01最新检查点：S11 release / 发布验收的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **11 / 17条产品路由**，S12–S17六条尚未开始。完整运行时和跨页面端到端集成未完成。

- [Screens总计划](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S11发布验收：请求、报告、证据与真实发布边界](FIGMA_SCREEN_RELEASE.md)
- [S11全部节点与恢复状态](FIGMA_SCREEN_RELEASE_STATE.json)
- [S10规则与模型](FIGMA_SCREEN_RULES.md) / [独立状态](FIGMA_SCREEN_RULES_STATE.json)
- [S09 Excel工作包](FIGMA_SCREEN_WORKBOOKS.md) / [独立状态](FIGMA_SCREEN_WORKBOOKS_STATE.json)
- [S08阵容预设](FIGMA_SCREEN_LINEUP_PRESETS.md) / [独立状态](FIGMA_SCREEN_LINEUP_PRESETS_STATE.json)
- [S07球员](FIGMA_SCREEN_PLAYERS.md) / [独立状态](FIGMA_SCREEN_PLAYERS_STATE.json)
- [S06球队](FIGMA_SCREEN_TEAMS.md) / [独立状态](FIGMA_SCREEN_TEAMS_STATE.json)
- [S05运行记录](FIGMA_SCREEN_RUNS.md) / [独立状态](FIGMA_SCREEN_RUNS_STATE.json)
- [S04赛后复盘](FIGMA_SCREEN_REVIEW.md) / [独立状态](FIGMA_SCREEN_REVIEW_STATE.json)
- [S03赛事推演](FIGMA_SCREEN_PREDICTION.md) / [独立状态](FIGMA_SCREEN_PREDICTION_STATE.json)
- [S02比赛与阵容](FIGMA_SCREEN_LINEUPS.md) / [独立状态](FIGMA_SCREEN_LINEUPS_STATE.json)
- [S01数据总览](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S12 `analytics` / 分析，尚未开始。** S11有33个产品状态、53个弹窗和1个Review Hub，仍只计一条路由。53个弹窗中40个为报告专属证据，只有2个运行确认；S01–S11累计348个状态，不是348条路由。

## S11本轮验收

总览、全链路、性能、安全、成本、历史及右侧报告摘要完成。A/B/C报告与39项证据、3个完整JSON示例保持身份与13项合计一致；另有独立D零预算成本证据。没有把新请求等待期间的旧A当作本次结果，也不在B读取失败时显示A。

**当前后端固定包含外部模型运行时Warning。** 页面旧fixture文案不代表真实模型执行；本设计没有伪造整体Pass，没有新增实际部署按钮。报告生成、持久化、发布许可和部署分别说明。无P95显示“—”，未运行不记0阻断；空预算、零预算和超预算独立，最新用量日期不等于今日实时账单。

Review Hub `599:136849` 可达33状态与53弹窗。最终 **676条NAVIGATE/OVERLAY、159项CLOSE、0其他动作**；S10默认Light/Dark的2条真实入站另计，只保留代表性主题，不转移候选规则包、比赛或许可。

深层9,941个未隐藏节点、3,796个真实嵌套实例；非预期越界、失效引用/目标、未绑定可见solid颜色、顶层重叠、异常按钮高度和证据身份错误均0。196个禁用/忙控件无反应，父级点击旁路0；22处正常纵向滚动。普通文字最低对比度Light4.5048、Dark6.8625。

158处本地文字对比度和62个继承CHANGE_TO演示反应已修正。2个最终运行确认没有执行连接；没有运行真实验收、数据库写入、模型、成本API或发布。所有报告ID、数据和占位哈希为设计演示，不能用于完整性证明。

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

底层组件与六组Patterns全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S11复用真实实例，没有创建Screen主组件、detach或修改共享资产和应用源码。

## 历史与运行时边界

各页原计数、节点、限定验收及未实现事项以独立状态为准，不能因S11通过而改写为全产品完成。S10四项客户端差异仍需落实：绑定赛事可选/必选不一致、P4路径过滤差异、同key版本保留顺序、浅JSON与候选失效。S09提交状态保护实现待办保留。

继续保留S02双方11首发与准入独立；S03输入/提供器/输出独立；S04查看/执行与复盘/结算分开；S05隐藏不删血缘；S06归档/删除/强清与导入/P4资格分开；S07身份、可用性、能力与短期标签独立；S08预设与比赛副本独立；S09类别/文件/模式/批次隔离；AI Chat只读上下文与session/request隔离。

S11任意参数和报告组合、真实Windows/PostgreSQL测量、模型与凭据验证、成本聚合、不可变账本事务/指纹核验、幂等与结果未知恢复、全部主题/折叠/焦点/滚动/草稿持续性尚未实现。部分刷新和类别操作显示规范，不是完整控制器；有限导航不证明跨页竞态正确。

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_RELEASE_STATE.json`及S11记录 → 实际稳定根。虚拟SLOT节点按根、语义名称和真实组件引用解析，不猜ID。

S10总状态保留于 `aeeee7700b5665566729da826e6ab2b0a81b39ae`；S09于 `4627fecbe7d83ce377c36db82d63aec2bed34712`；更早恢复链见Screens计划和独立记录。初始/组合WIP已被accepted覆盖。旧截图、空handoff、旧“Screens未开始”不能代替最新明确状态；未改组件API不为更新进度重写。
