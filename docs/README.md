# Figma Design System Docs

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支 `ui-design-system`。

## 当前恢复入口

**2026-10-01最新检查点：S12 analytics / 分析的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **12 / 17条产品路由**，S13–S17五条尚未开始。真实运行时和完整跨页面端到端集成未完成。

- [Screens总计划](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S12分析：任务、质量、审核、H/I与参数边界](FIGMA_SCREEN_ANALYTICS.md)
- [S12全部节点与恢复状态](FIGMA_SCREEN_ANALYTICS_STATE.json)
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

**下一项：S13 `api_workspace` / AI问答，尚未开始。** S12有50个产品状态、48个弹窗和1个Review Hub；仍只计一条路由。S01–S12累计398个状态，不是398条路由。

## S12本轮验收

历史样本与H监控、完整分析与后台任务、质量检查/人工审核、受控参数生命周期完成；含外部分析回包和高级统计。当前公开参数生成接口直接拒绝未捆绑提供器，生成按钮禁用；候选/影子/晋升/回滚只是条件化历史示例，不冒充当前生成。

正式结算与评估样本口径分开，最近全局快照不代表H/I精确分区许可。无scan_id不按0问题通过；取消请求不等于取消终态；结果未知不自动重发。回包导入、建议接受、pending能力候选、能力历史写入分别处理，绑定改变禁止旧候选晋升。

Review Hub `614:153064` 可达50状态与48弹窗。最终 **937条NAVIGATE/OVERLAY、144项CLOSE、0其他动作**，外部4条入站另计：S01/S11默认Light/Dark的分析导航进入真实S12，只保留代表性主题，不转移比赛、任务、文件、分区或许可。

最终13,938个未隐藏节点、5,424个真实嵌套实例；非预期越界、失效引用/目标、未绑定solid颜色、顶层重叠、异常按钮高度与门禁断言错误均0。266个禁用/忙控件无反应，父级旁路0；15处正常纵向滚动。普通文字最低对比度Light4.5327、Dark6.8625。

修复Steps SLOT纵向挤占正文，改为真实SLOT内的原生横向布局并保留四个Step实例；修正220处指标说明对比度、9处结果/扫描文案和未连接服务状态。四种证据判定分别保留必填说明字段。**25个最终操作确认均无执行连接，没有运行分析、写库、导入文件或改变参数。**

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

底层组件与六组Patterns全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S12没有新增Screen主组件、detach、修改共享资产或应用源码。

## 历史与运行时边界

以前各页计数、节点、验收和未实现事项继续以其独立状态为准。S12没有重新验收全部旧页，也没有消除S01–S11运行时缺口。

S02双方11首发与准入独立；S03输入/提供器/输出独立；S04查看/执行、复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强清和导入/P4门槛分开；S07来源身份、长期能力/短期标签独立；S08预设与比赛副本分离；S09类别/文件/模式/批次独立；S10读取/校验/注册/绑定/执行分开；S11报告完成不等于发布通过，保留公开运行时警告。AI Chat维持只读上下文主动勾选及session/request隔离。

S12任意参数与分区、真实任务及结果未知恢复、H评分、质量事实修复、回包文件身份/幂等、审核说明、能力历史事务、私有提供器、影子样本及绑定重检、人工晋升/回滚、所有权限/主题/滚动/草稿/焦点持续性尚未实现。表格只是代表性子集，诊断/刷新/字段入口部分为说明，不是完整控制器。设计验收不代表源码差异已修复或后端全面审计。

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_ANALYTICS_STATE.json`及S12记录 → 实际Figma稳定根。虚拟SLOT后代按根、语义名称和真实组件引用解析，不猜ID。

S11总检查点保留于 `a1ede69ac02f7ab7ac73e73485a44cc4e97f7eab`；S10于 `aeeee7700b5665566729da826e6ab2b0a81b39ae`；S09于 `4627fecbe7d83ce377c36db82d63aec2bed34712`；更早恢复链见Screens计划。初始/组合WIP不能覆盖最新accepted。旧截图、空handoff和旧“Screens未开始”记录不能替代明确检查点。
