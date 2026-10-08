# Figma Design System Docs

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支 `ui-design-system`。

## 当前恢复入口

**2026-10-08最新检查点：S14 openai / 兼容API配置的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **14 / 17条产品路由**，S15–S17三条尚未开始。真实运行时和完整跨页面端到端集成未完成。

- [Screens总计划](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S14兼容API：配置、密钥、解析、测试与恢复边界](FIGMA_SCREEN_OPENAI.md)
- [S14全部节点与恢复状态](FIGMA_SCREEN_OPENAI_STATE.json)
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

**下一项：S15 `database` / 数据库，尚未开始。** S14有38个产品状态、24个弹窗和1个Review Hub，仍只计一条路由。S01–S14累计486个状态，不是486条路由。

## S14本轮验收

完成配置档案、请求与密钥、模型与研究、测试与安全四个区域，含API Example预置解析、参数校验、保存/测试/结果未知、配置A/B隔离、激活、密钥移除和删除确认。

选中、当前、密钥存在、保存完成与连接探测成功分别呈现。未测试但有密钥的B可以打开激活确认，这不表示连接通过。Responses仅为正式研究候选；一次最小探测不验证全部工具、结构化输出或来源元数据。A的测试不显示到B，r1响应不能验证r2草稿。

保存密钥不会回显，显隐只作用于本次输入；留空、替换和移除分别处理。元数据与凭据不是同一原子事务，结果未知先核验。删除配置不改绑旧会话。业务数据库未连接不阻止本机设置管理。

Review Hub `656:178986`可达38状态与24弹窗。最终 **1065条NAVIGATE/OVERLAY、72项CLOSE、0其他动作**，外部4条入站另计：S13新会话Light/Dark的侧栏和工具栏进入真实S14，仅保留代表性主题，不转移会话、草稿、密钥或许可。

最终8894个未隐藏节点、3465个实例、2399个文字层；256个禁用控件自身反应和父级旁路均0。非预期越界、失效主组件/目标、未绑定solid颜色、顶层重叠、异常按钮高度和状态断言错误均0；9处正常滚动。2278普通文字层测量，最低对比度Light4.5048/Dark7.0938。

修正协议说明、测试摘要和侧栏一致性、完成状态的导航保护，同帧导航保持无动作。删除/移除使用现有Danger语义；长表单保存栏在滚动区外。**12个最终操作确认均无执行连接，没有实际保存凭据、发起网络探测、激活/删除配置或修改数据库。** 测试与解析结果为明确预置示例。

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

底层组件与六组Patterns全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S14没有新增Screen主组件、detach、修改共享资产或应用源码。

## 历史与运行时边界

此前各页计数、节点、验收与未实现事项继续以独立状态为准；S14没有重新验收旧页或消除S01–S13运行时缺口。S13历史验收仍为50状态/24弹窗，原会话和请求隔离不因后续接线而放宽。

S02双方11首发与准入独立；S03输入/提供器/输出独立；S04查看/执行与复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强清与导入/P4分开；S07来源身份及长期能力/短期标签独立；S08预设与比赛副本分离；S09类别/文件/模式/批次隔离；S10读取/校验/注册/绑定/执行分开；S11报告完成不等于发布；S12全局快照不代表精确分区许可，建议接受不等于能力历史或正式参数已修改；S13只读上下文主动勾选及session/request/草稿隔离继续有效。

S14任意参数、完整协议/模型组合、真实Windows凭据操作、网络收费和服务能力验证、保存失败/回滚与结果未知核验、异步解析版本归属、测试快照匹配、日志脱敏及完整主题/滚动/焦点/草稿持续性仍需实现。特别是当前update_test_state按profile_id回写，设计的连接版本保护仍需落实；没有把设计通过记成源码已修复。

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_OPENAI_STATE.json`与S14记录 → 实际Figma稳定根。SLOT虚拟后代按稳定根、语义名称和主组件解析，不猜ID。

S13完整总检查点保留于 `9ae16eaf438d3a8963fd2868e9b26611fd963dd4`；S12于 `0cdcbf5fd4c878f7cb4a8995786677f8ea251263`；S11于 `a1ede69ac02f7ab7ac73e73485a44cc4e97f7eab`；S10于 `aeeee7700b5665566729da826e6ab2b0a81b39ae`；S09于 `4627fecbe7d83ce377c36db82d63aec2bed34712`；更早恢复链见Screens计划。

传输中断后先回读实际画布，不重建已有节点。初始/组合WIP不能覆盖accepted；旧截图、空handoff和旧“Screens未开始”不能替代最新明确检查点。Create State handoff未创建；以GitHub独立记录及同步总检查点恢复。S15尚未开始。
