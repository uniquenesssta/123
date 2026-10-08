# Figma Design System Docs

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支`ui-design-system`。

## 当前恢复入口

**2026-10-08最新检查点：S16 logs / 问题日志的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前**16 / 17条产品路由**，只剩S17架构信息尚未开始。真实运行时和完整跨页面端到端集成未完成。

- [Screens总计划](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S16问题日志：历史、聚合、导出和清空边界](FIGMA_SCREEN_LOGS.md)
- [S16全部节点与恢复状态](FIGMA_SCREEN_LOGS_STATE.json)
- [S15数据库](FIGMA_SCREEN_DATABASE.md) / [独立状态](FIGMA_SCREEN_DATABASE_STATE.json)
- [S14兼容API](FIGMA_SCREEN_OPENAI.md) / [独立状态](FIGMA_SCREEN_OPENAI_STATE.json)
- [S13 AI问答](FIGMA_SCREEN_API_WORKSPACE.md) / [独立状态](FIGMA_SCREEN_API_WORKSPACE_STATE.json)
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

**下一项：S17 `architecture` / 架构信息，尚未开始。** S16有30个产品状态、12个弹窗和1个Review Hub，仍只计一条路由。S01–S16累计548个状态，不是548条路由。

## S16本轮验收

完成问题概览、聚合机制、记录与技术详情、读取/刷新/失败/空、无数据库、后台扫描失败、重复任务尝试、严重度升级、导出快照、清空失败/未知及再次出现的代表性状态。

**历史问题不等于故障仍在发生；清空日志不等于修复故障。** 独立问题、发生次数、重复次数和严重问题条数分别计数。基准3条/11次/8重复/1严重；同次任务扫描不增量，新尝试可增量；保留上限不是无限去重保证。

**本机日志可在无数据库时使用。** 连接数据库后的刷新可补充失败任务，扫描失败也可成为本机问题。空、未知和读取失败分开；刷新保留旧快照。报告是导出时刻的本地文本快照，不上传，不自动更新，也不保证包含所有历史或完整运行追踪。

**当前清空实现先改内存再落盘。** 失败不能保证旧内存/文件仍在，未知结果先核验，不自动重发；清空后再次扫描可重新产生同指纹记录，从新计数周期开始。清空不影响业务数据库和独立运行追踪文件。

Review Hub `686:195370`可达30状态和12弹窗。最终**150 NAVIGATE +336 OVERLAY =486，0 SWAP、36 CLOSE、其他0**；4条S15入站另计，默认Light/Dark保留主题，恢复/重建失败入口进入无数据库的本机日志，不虚构健康状态。

检查7005个可见节点、2562个实例、2032个文字层；非预期越界、失效引用/目标、未绑定颜色、顶层重叠、异常按钮和计数门禁错误均0。145个禁用控件自身反应和祖先旁路均0。2007普通文字层最低对比度Light4.5048/Dark6.5682，20处本地Warning文字修正。11处实际超高滚动，长技术详情540px viewport/1089px内容，报告关闭栏保持在滚动区之外；没有测试完整500条极限。

**2个最终操作示意及1个集合变化禁用确认的最终按钮均无执行连接。** 没有实际采集、导出、清空日志、重试业务或修复数据库。导出保存是原生位置选择的有限示意，不是新增的强制业务确认；结果由明确标注的演示入口选择。

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

底层组件与六组Patterns全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S16沿用该资产清单，没有新增Screen主组件、detach、修改共享资产或应用源码。

## 历史与运行时边界

此前各页计数、节点、验收与未实现事项继续以独立状态为准；S16不重新验收旧页，也不消除S01–S15运行时缺口。S15原32状态/16弹窗、S14原38/24与S13原50/24的独立记录保持，后续入站不放宽既有门禁。

S02双方11首发与准入独立；S03输入/提供器/输出独立；S04查看/执行与复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强清与导入/P4分开；S07来源身份及长期能力/短期标签独立；S08预设与比赛副本分离；S09类别/文件/模式/批次隔离；S10读取/校验/注册/绑定/执行分开；S11报告完成不等于发布；S12全局快照不代表精确分区许可，建议接受不等于能力历史或正式参数已修改；S13只读上下文主动勾选及session/request/草稿隔离；S14配置选中/当前/密钥存在/测试独立，旧测试不能验证新连接草稿。

S15连接并保存包含迁移，清除连接不删业务，schema删除先提交再重建，重建失败不证明数据恢复。S14测试快照、凭据事务、协议能力、脱敏与持续性；S15输入一致性、完整目标版本、跨文件恢复、任务/外部客户端并发、URL全路径脱敏和外部恢复准备，继续保留为实现待办。

S16还需处理初始化读取/解析错误被吞为空集合、内存与文件写入不原子、clear缺少集合版本参数及定向脱敏不完整。任意记录/原生文件选择/并发采集/文件失败恢复/完整敏感信息审查和主题滚动焦点持续性没有实现。设计通过不等于源码已修复或完整安全审计。

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_LOGS_STATE.json`与S16记录 → 实际Figma稳定根。SLOT虚拟后代按稳定根、语义名称和主组件解析，不猜ID。

S15同步总检查点保留于`920432210478b26e17f2a3c9ffaf74c0f9f01e6f`；S14于`52236b78b5a0f478bb6fb3b4235d30fd92d4e97c`；S13于`9ae16eaf438d3a8963fd2868e9b26611fd963dd4`；S12于`0cdcbf5fd4c878f7cb4a8995786677f8ea251263`；S11于`a1ede69ac02f7ab7ac73e73485a44cc4e97f7eab`；S10于`aeeee7700b5665566729da826e6ab2b0a81b39ae`；S09于`4627fecbe7d83ce377c36db82d63aec2bed34712`；更早恢复链见Screens计划。

失败批次按工具回滚标记恢复，不重复创建。初始WIP不能覆盖accepted；旧截图、空handoff和旧“Screens未开始”不替代明确检查点。Create State handoff未创建；恢复依据为GitHub独立记录及同步总检查点。S17尚未开始。
