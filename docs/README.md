# Figma Design System Docs

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支 `ui-design-system`。

## 当前恢复入口

**2026-10-02最新检查点：S13 api_workspace / AI问答的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **13 / 17条产品路由**，S14–S17四条尚未开始。真实运行时与完整跨页面端到端集成未完成。

- [Screens总计划](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S13 AI问答：会话、请求、上下文与归档边界](FIGMA_SCREEN_API_WORKSPACE.md)
- [S13全部节点与恢复状态](FIGMA_SCREEN_API_WORKSPACE_STATE.json)
- [S12分析](FIGMA_SCREEN_ANALYTICS.md) / [独立状态](FIGMA_SCREEN_ANALYTICS_STATE.json)
- [S11发布验收](FIGMA_SCREEN_RELEASE.md) / [独立状态](FIGMA_SCREEN_RELEASE_STATE.json)
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

**下一项：S14 `openai` / 兼容API配置，尚未开始。** S13有50个产品状态、24个弹窗和1个Review Hub；仍只计一条路由。S01–S13累计448个状态，不是448条路由。

## S13本轮验收

完成对话工作台、历史、搜索/读取/刷新、检查器、球队/球员/比赛只读上下文、A/B/NEW请求与草稿隔离、发送等待/失败/取消/结果未知、历史文件审计及会话归档确认。

客户端保存会话与消息不等于AI可修改业务资料。上下文默认不附加，主动勾选才用于下一条；既有消息的摘要快照不被下一条勾选改写。已保存会话锁定原配置、类型和对象，缺少密钥不自动换配置。历史C的SQL、HTML、文件与提案只读，不恢复上传/生成/执行。

A发送期间可以浏览B，B正文没有A待处理消息，B草稿不变且不能绕过发送锁。A完成仍停留B，返回A才看见A新增消息。B读取失败不显示A；取消待确认、取消终态与结果未知分开。失败保留最近确认的历史，不据此断言服务端没有保存新用户消息。

Review Hub `635:166265`可达50状态与24弹窗。最终 **1196条NAVIGATE/OVERLAY、72项CLOSE、0其他动作**；外部4条入站另计，S01/S12默认Light/Dark的AI主导航进入真实S13，只保留代表性主题，不转移比赛、分析、文件、草稿或许可。

深层11615个未隐藏节点、4682个真实嵌套实例；非预期越界、失效引用/目标、未绑定solid颜色、顶层重叠、异常按钮高度和会话断言错误均0。384个禁用控件无点击反应，父级旁路0；4处正常纵向滚动。2658个普通文字层测量，最低对比度Light4.5327/Dark6.8625。

修正历史tab双高亮、数据库服务状态、96处局部文字对比度和紧凑布局。长消息在320高视口内滚动，输入区始终在外。普通发送只进入模拟等待，结果由明确的人工演示入口选择；4个归档最终确认没有执行连接。**没有真实API、数据库、凭据、剪贴板、上传或文件生成操作。**

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

底层组件与六组Patterns全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S13没有新增Screen主组件、detach、修改共享资产或应用源码。

## 历史与运行时边界

此前各页计数、节点、验收与未实现事项继续以独立状态为准；S13没有重新验收全部旧页或消除S01–S12运行时缺口。

S02双方11首发与准入独立；S03输入/提供器/输出独立；S04查看/执行与复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强清与导入/P4分开；S07来源身份及长期能力/短期标签独立；S08预设与比赛副本分离；S09类别/文件/模式/批次隔离；S10读取/校验/注册/绑定/执行分开；S11报告完成不等于发布，保留公开运行时警告；S12全局快照不代表精确分区许可，接受建议不等于能力历史或正式参数已修改。

S13任意输入、所有配置/类型/对象组合、全量会话查询、真实请求及账本幂等、取消/重试与结果未知恢复、全量转义/权限/剪贴板验证、所有会话草稿和主题/折叠/滚动/来源/焦点持续性仍需实现。部分控件显示范围说明，不能把有限原型当作完整控制器。历史后端类型的保留不意味着本页面恢复那些执行能力。

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_API_WORKSPACE_STATE.json`及S13记录 → 实际Figma稳定根。虚拟SLOT后代按根、语义名称和真实组件引用解析，不猜ID。

S12总检查点保留于 `0cdcbf5fd4c878f7cb4a8995786677f8ea251263`；S11于 `a1ede69ac02f7ab7ac73e73485a44cc4e97f7eab`；S10于 `aeeee7700b5665566729da826e6ab2b0a81b39ae`；S09于 `4627fecbe7d83ce377c36db82d63aec2bed34712`；更早恢复链见Screens计划。S12曾补齐滞留总计划的同步，本轮独立记录与三处索引必须一致。

S13传输中断后已回读实际接线，不重建页面。初始/组合WIP不能覆盖accepted；旧截图、空handoff和旧“Screens未开始”记录不能替代最新明确检查点。Create State handoff未创建，GitHub明确记录为恢复依据。
