# Figma Design System Docs

**2026-10-09 第二批修复检查点：INT-01 已修复；INT-03 的球队来源档案返回与 INT-02 的普通导航主题连续性已通过存储动作复验。705 个产品状态的普通主题变化候选为 0；28 个明暗来源档案保持首尔 FC 返回。整体仍为 NOT_PASSED，INT-04 待修复及全产品复验。未进行浏览器全事件回放或真实运行时验收。详见[修复记录](FIGMA_PROTOTYPE_INTEGRATION_REPAIRS.md)和[本批节点与复验数据](FIGMA_PROTOTYPE_REPAIR_BATCH02.json)。**

> 下文原验收的 933 根、561 状态、156 候选及“只读／尚未修复”均为历史快照；当前为 1,077 根、705 状态，INT-01/02/03 限定范围已修复，INT-04 未修复。当前状态以顶部、修复记录及第二批数据为准。

按需读取各层级独立记录，不把全部设计合成一份大文档。

**设计依据：尺寸、比例、间距、字号和颜色以Figma为准；源码只提供功能、交互和数据语义。** 唯一设计记录分支`ui-design-system`。

## 当前恢复入口

**2026-10-09：INT-01、INT-02、INT-03 的限定存储动作修复已完成；整体仍未通过，下一步修复 INT-04 并复验。** 17 / 17条产品页面的可编辑视觉及限定本页状态仍已完成，没有缺失页面，也没有新增S18。

最新结果与恢复入口：

- [原型集成修复记录](FIGMA_PROTOTYPE_INTEGRATION_REPAIRS.md)
- [第二批节点、主题映射与复验数据](FIGMA_PROTOTYPE_REPAIR_BATCH02.json)

- [全产品原型验收报告、问题节点与复验条件](FIGMA_PROTOTYPE_INTEGRATION_AUDIT.md)
- [本次验收状态与逐页统计](FIGMA_PROTOTYPE_INTEGRATION_STATE.json)
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

## 首次全产品原型验收结果（修复前历史）

本次只读检查933个根（561状态、354弹窗、1菜单、17验收目录）及12,215项存储反应。各页验收目录均可达自己的全部状态；从首页不用验收捷径也能到达17条路由。失效目标与NAVIGATE/OVERLAY/SWAP误入Screens外目标均0；3,738个可见Disabled实例自身和祖先指针旁路均0。

**整体仍未通过：** 默认Light/Dark共34页中92个非选中导航没有点击动作、4个仅显示说明；普通导航存在156条主题变化静态候选；球队→球员来源档案→基础资料会丢失来源球队返回入口（仍是A01，不是串成其他球员）；全局返回/前进、折叠/重置与完整状态保持仍未形成统一可验收行为。

问题编号INT-01至INT-04已记录准确节点、范围和复验条件。156是静态候选数量，不是156个逐一回放的独立缺陷；92/4只统计默认明暗页面。检查未执行浏览器全事件回放、真实请求或写入，也没有改动画布或应用源码。下一步修复原型问题后复验，不能把本轮“验收执行结束”记为“集成已通过”。

S17原有局部结果保留：13个阅读/布局状态、5个只读说明弹窗和1个Review Hub，只计一条路由。S01–S17累计561个状态，不是561条路由。

## S17历史验收

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

底层组件、六组Patterns和17条Screens可编辑视觉阶段完成。Controls **61 sets/506 variants**，Patterns **17 sets/125 variants/13 production singles**；Foundations在S17回读仍为**193 variables/13 text styles/6 effects**。本次原型验收没有修改共享资产或应用源码。

## 历史与后续边界

以前各页的详细节点、有限验收与未实现事项继续以独立记录为准。S17不重新验收S01–S16，不消除历史运行时缺口；本轮整体验收也没有重做全部视觉/字体/对比度检查。

S02双方11首发与准入独立；S03输入/提供器/输出独立；S04查看/执行与复盘/结算分开；S05隐藏不删血缘；S06归档/普通删除/强清与导入/P4分开；S07来源身份、长期能力和短期标签独立；S08预设与比赛副本分离；S09类别/文件/模式/批次隔离；S10读取/校验/注册/绑定/执行分开；S11报告完成不等于发布；S12全局快照不是精确分区许可；S13只读上下文主动勾选和会话/请求/草稿隔离；S14选中/当前/密钥/测试独立；S15schema删除先提交再重建，失败不承诺回滚；S16历史日志不是当前故障，清空不修复业务。

后续处理 INT-04 并复验完整导航、主题/折叠/返回/滚动/焦点/草稿持续性；INT-01/02/03 已完成上述限定范围修复。实际文件和API请求、凭据/数据库事务、异步失效保护、模型可用性、真实发布验收及各页源码差异仍为独立实现工作。**视觉完成、原型集成通过与运行时/安全/发布完成分别记录。**

## 换会话恢复

README → `FIGMA_PROTOTYPE_INTEGRATION_REPAIRS.md` → `FIGMA_PROTOTYPE_REPAIR_BATCH02.json` → `FIGMA_PROTOTYPE_INTEGRATION_STATE.json` → `FIGMA_SCREENS_STATE.json`中的integrationRepair → 实际Figma稳定根。各页独立状态继续作为局部依据；SLOT后代按稳定根、语义名称和真实主组件定位，不猜ID。

S17视觉阶段完整基线为`67e2c5000dce035b70860826c55ccde61fe84396`；S16为`7d3b72310a745a007075d3b8ed35ef38cc33a49d`，S15为`920432210478b26e17f2a3c9ffaf74c0f9f01e6f`，S14为`52236b78b5a0f478bb6fb3b4235d30fd92d4e97c`；更早历史链保留在Screens计划和Git历史。

初始WIP、回滚批次、旧截图和旧“未开始”不覆盖accepted。Create State handoff未创建；恢复依据是GitHub独立记录及同步总检查点。原型验收已执行且未通过；INT-01/02/03 限定修复已完成，INT-04 和后续源码实现尚未完成。
