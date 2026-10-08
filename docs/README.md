# Figma Design System Docs

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支`ui-design-system`。

## 当前恢复入口

**2026-10-08最新检查点：S15 database / 数据库的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前**15 / 17条产品路由**，S16–S17两条尚未开始。真实运行时和完整跨页面端到端集成未完成。

- [Screens总计划](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S15数据库：连接、自动迁移、统计与重置边界](FIGMA_SCREEN_DATABASE.md)
- [S15全部节点与恢复状态](FIGMA_SCREEN_DATABASE_STATE.json)
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

**下一项：S16 `logs` / 问题日志，尚未开始。** S15有32个产品状态、16个弹窗和1个Review Hub，仍只计一条路由。S01–S15累计518个状态，不是518条路由。

## S15本轮验收

完成运行状态、连接设置、核心统计、技术信息和危险重置，覆盖候选连接/自动迁移、本机保存失败、恢复失败、清除连接、统计刷新/未知/空业务及重置重建失败、重连失败和结果未知。

**连接并保存包含自动迁移，不是只读测试。** 草稿B和活动A分开；尚未确认的B不借用A健康或统计。清除本机连接不删除服务器数据。凭据与元数据分步写入，失败不证明全程回滚。

**重置schema删除事务先提交，再重建。** 后续迁移或重连失败不能宣称旧数据已恢复；结果未知禁止自动破坏性重试。名称空/错误/匹配/目标变化分别呈现，完整服务身份和连接版本仍需运行时绑定。数据库本身和本机连接保留，重新注册系统资料不等于所有表为0。客户端没有备份/还原入口，不虚构已完成备份。

Review Hub `676:187457`可达32状态和16弹窗。最终**135条NAVIGATE、393条OVERLAY（合计528）、9条人工确认状态SWAP、32项CLOSE，其他0**。外部4条入站另计：S01 Light/Dark进入同主题概览；S13/S14无数据库入口进入未配置页，不误入健康状态。

最终6935个未隐藏节点、2744个实例、1732个文字层；非预期越界、失效主组件/目标、未绑定solid颜色、顶层重叠、异常按钮高度、门禁错误均0。186禁用控件自身反应和祖先旁路均0。1678普通文字层测量，最低对比度Light4.5048/Dark6.8574；72处局部对比度及重置完成页当前tab接线已修正。

**3个最终操作确认和3个禁用重置门禁的最终按钮全部无执行连接。** 没有实际连接、迁移、保存凭据、清除、备份或重置。结果均为人工预置。正文viewport与外置动作栏结构已核对；本组实际内容超高滚动区域为0，不宣称完成长内容滚动压力测试。

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

底层组件与六组Patterns全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S15没有新增Screen主组件、detach、修改共享资产或应用源码。

## 历史与运行时边界

此前各页计数、节点、验收与未实现事项继续以独立状态为准；S15不重新验收旧页，也不消除S01–S14运行时缺口。S14原38状态/24弹窗与S13原50状态/24弹窗的独立记录保持，后续入站不放宽既有门禁。

S02双方11首发与准入独立；S03输入/提供器/输出独立；S04查看/执行与复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强清与导入/P4分开；S07来源身份及长期能力/短期标签独立；S08预设与比赛副本分离；S09类别/文件/模式/批次隔离；S10读取/校验/注册/绑定/执行分开；S11报告完成不等于发布；S12全局快照不代表精确分区许可，建议接受不等于能力历史或正式参数已修改；S13只读上下文主动勾选及session/request/草稿隔离；S14配置选中/当前/密钥存在/测试独立，旧测试不能验证新连接草稿。

S14 update_test_state连接快照匹配、凭据事务恢复、协议能力、日志脱敏与完整持续性仍需实现。S15未知值/输入上下限一致性、完整重置目标版本、跨文件保存恢复、schema清空后重建失败、任务/外部客户端并发、URL全路径脱敏及外部恢复准备同样未修复。设计通过不等于源码已修复或完整安全审计。

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_DATABASE_STATE.json`与S15记录 → 实际Figma稳定根。SLOT虚拟后代按稳定根、语义名称和主组件解析，不猜ID。

S14同步总检查点保留于`52236b78b5a0f478bb6fb3b4235d30fd92d4e97c`；S13于`9ae16eaf438d3a8963fd2868e9b26611fd963dd4`；S12于`0cdcbf5fd4c878f7cb4a8995786677f8ea251263`；S11于`a1ede69ac02f7ab7ac73e73485a44cc4e97f7eab`；S10于`aeeee7700b5665566729da826e6ab2b0a81b39ae`；S09于`4627fecbe7d83ce377c36db82d63aec2bed34712`；更早恢复链见Screens计划。

传输中断后按工具回滚标记和实际画布恢复，不重复创建。初始WIP不能覆盖accepted；旧截图、空handoff和旧“Screens未开始”不替代明确检查点。Create State handoff未创建；恢复依据为GitHub独立记录及同步总检查点。S16尚未开始。
