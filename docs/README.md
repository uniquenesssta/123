# Figma Design System Docs

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支`ui-design-system`。

## 当前恢复入口

**2026-10-09最新检查点：S17 architecture / 架构信息的可编辑视觉、代表性阅读状态与有限原型已验收。Screens计划内17 / 17条产品页面均完成这一层验收，没有尚未制作的页面。**

**这不等于软件已全部实现或可发布。全产品导航、全部状态持续性、真实功能和端到端集成仍待单独验收。** S17之后没有擅自新建S18或启动实现阶段。

- [Screens总计划与后续边界](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S17架构信息：系统边界、原则与模块职责](FIGMA_SCREEN_ARCHITECTURE.md)
- [S17全部稳定节点与恢复状态](FIGMA_SCREEN_ARCHITECTURE_STATE.json)
- [S16问题日志](FIGMA_SCREEN_LOGS.md) / [独立状态](FIGMA_SCREEN_LOGS_STATE.json)
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

S17有13个阅读/布局状态、5个只读说明弹窗和1个Review Hub，仍只计一条路由。S01–S17累计561个状态，不是561条路由。

## S17本轮验收

完成四层职责、历史与路由原则、五类模块展开/收起、明暗主题、Compact及无数据库阅读。静态说明不伪装实时健康、运行版本、已安装模型或发布通过；没有增加无关的加载、保存、删除或执行流程。

正常业务的历史修订原则不等于记录永不删除或已完成备份。自动绑定仍需有效、启用并匹配请求上下文，没有匹配不补造默认。模型接口与外部计算运行时独立；源码工作区版本不冒充正在运行的客户端版本。

Review Hub `694:197834`可达全部13状态和5弹窗。最终**165 NAVIGATE +66 OVERLAY =231条连接，10 CLOSE，其他0**。外部6条S15/S16入站另计：默认Light/Dark进入对应主题；无数据库/恢复失败进入静态无数据库阅读，不转移凭据、日志、修复对象或权限。

深层检查2484个未隐藏节点、900个嵌套实例、598个文字层；越界、失效主组件/目标、未绑定颜色、顶层重叠、异常按钮和静态断言错误均0。52个禁用控件自身反应/祖先旁路0。18处局部对比度修正后，普通文字最低Light4.5327/Dark6.8625。5处实际超高阅读区域，底部验收栏在滚动区之外。

本页所有弹窗只解释和关闭，**没有执行真实网络、文件、模型、凭据或数据库操作**。Shell折叠/重置及未覆盖Compact主题组合显示范围说明，不冒充完整控制器。17个真实产品默认根存在已回读，但不是重新验收全部旧页或完成全产品集成。

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

底层组件、六组Patterns和17条Screens可编辑视觉阶段完成。Controls **61 sets/506 variants**，Patterns **17 sets/125 variants/13 production singles**；Foundations本轮回读仍为**193 variables/13 text styles/6 effects**。S17没有新增Screen主组件、detach或修改共享资产和应用源码。

## 历史与后续边界

以前各页的详细节点、有限验收与未实现事项继续以独立记录为准。S17不重新验收S01–S16，不消除历史运行时缺口。

S02双方11首发与准入独立；S03输入/提供器/输出独立；S04查看/执行与复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强清与导入/P4分开；S07来源身份、长期能力和短期标签独立；S08预设与比赛副本分离；S09类别/文件/模式/批次隔离；S10读取/校验/注册/绑定/执行分开；S11报告完成不等于发布；S12全局快照不是精确分区许可；S13只读上下文主动勾选和会话/请求/草稿隔离；S14选中/当前/密钥/测试独立；S15schema删除先提交再重建，失败不承诺回滚；S16历史日志不是当前故障，清空不修复业务。

后续须单独处理完整导航与目标一致性、主题/折叠/返回/滚动/焦点/草稿持续性、实际文件和API请求、凭据/数据库事务、异步失效保护、模型可用性、真实发布验收，以及各页已经记录的源码差异。**视觉完成与运行时/安全/发布完成分别计数。**

## 换会话恢复

README → `FIGMA_SCREENS_PLAN.md` → `FIGMA_SCREENS_STATE.json` → `FIGMA_SCREEN_ARCHITECTURE_STATE.json`及本页记录 → 实际Figma稳定根。SLOT后代按稳定根、语义名称和真实主组件定位，不猜ID。

S16完整检查点为`7d3b72310a745a007075d3b8ed35ef38cc33a49d`，S15为`920432210478b26e17f2a3c9ffaf74c0f9f01e6f`，S14为`52236b78b5a0f478bb6fb3b4235d30fd92d4e97c`；更早历史链保留在Screens计划和Git历史。

初始WIP、回滚批次、旧截图和旧“未开始”不覆盖accepted。Create State handoff未创建；恢复依据是GitHub独立记录及同步总检查点。后续全产品集成工作尚未开始。
