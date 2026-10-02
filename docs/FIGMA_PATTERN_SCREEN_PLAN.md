# Figma Patterns & Screen Plan

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互和数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支 `ui-design-system`。

## 当前状态

图标、底层组件与Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有12 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks、S10 rules、S11 release、S12 analytics。S13–S17五条尚未开始。** 累计398个产品状态仍只计12条路由；弹窗、Review Hub、上下文菜单和Pattern QA不增加路由数量。真实运行时与完整跨路由端到端集成未完成。

S12独立验收日期为2026-10-01；2026-10-02中断恢复时重新读取画布与独立记录，补齐此前滞留S11的总计划和检查点，不重建页面。恢复：[总状态](FIGMA_SCREENS_STATE.json) → [S12详细状态](FIGMA_SCREEN_ANALYTICS_STATE.json) → [S12独立记录](FIGMA_SCREEN_ANALYTICS.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S13 `api_workspace` / AI问答，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件与Composer。

各组独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat历史状态](FIGMA_AI_CHAT_STATE.json)和[Workflow历史状态](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)不能覆盖后续Screens进度。组件API未改变时，不为更新进度重写其历史记录。

## 当前阶段：Screens

- [x] S01 `dashboard`：总览、连接前置与代表性状态。
- [x] S02 `lineups`：比赛、双方阵容、模型链路、历史、局部工作包。
- [x] S03 `prediction`：正式推演、研究、历史、临时演练。
- [x] S04 `review`：九步复盘、补录、事件与候选。
- [x] S05 `runs`：独立运行列表、详情、技术追踪、历史隐藏。
- [x] S06 `teams`：目录、名单、档案、阵型/任期、资料包。
- [x] S07 `players`：目录、来源身份、档案、履历、状态/能力/标签、工作包。
- [x] S08 `lineup_presets`：按球队管理预设、编辑校验、复制/归档/删除、套用边界。
- [x] S09 `workbooks`：三类工作包、导出/预检/提交、类别与批次隔离。
- [x] S10 `rules`：赛事目录、层级、模型路由、规则包。
- [x] S11 `release`：验收请求、分类检查、报告证据、性能/安全/成本、历史。
- [x] S12 `analytics`：历史/H监控、分析任务、质量/人工审核、外部回包与受控参数生命周期。
- [x] 独立状态中明确列出的部分真实入站与返回。
- [ ] **S13 `api_workspace`：下一项，尚未开始。**
- [ ] S14–S17：见独立Screens计划，尚未开始。
- [ ] 其余真实目标、全状态持续性、完整跨路由与运行时联调。

历史S01–S11计数和验收含义保持于各自记录。S12新增50状态、48弹窗和Review Hub `614:153064`，只增加一条产品路由。

## S12验收与边界

历史样本与H监控、分析与任务、质量与人工审核、外部分析资料包及受控参数生命周期完成代表性设计。正式结算与分析有效样本口径分开；最近全局快照不等于H/I精确分区许可。无scan_id不按0问题通过；取消请求不等于取消终态；结果未知不自动重发。

能力建议接受只建立pending候选，能力历史独立审核。四种证据判定均保留必填说明；质量标记不修复业务事实。回包文件身份改变使预检失效，导入中/已导入禁重复提交。当前公开候选生成接口没有提供器，生成按钮禁用；候选、影子、晋升和回滚均为条件化历史示例，绑定改变禁止旧晋升。

独立验收记录为 **937条NAVIGATE/OVERLAY、144项CLOSE、0其他动作**，Hub可达50状态和48弹窗；4条S01/S11默认Light/Dark入站另计，仅保留代表性主题，不转移比赛、任务、文件、分区或许可。25个最终操作确认无执行连接，未实际运行任务、文件导入、能力写回或参数变更。

原深层视觉验收检查13,938个未隐藏节点、5,424嵌套实例、3,919普通文字层。非预期越界、失效引用/目标、未绑定solid颜色、顶层重叠、异常按钮高度和门禁断言错误均0；266禁用/忙控件反应及父级旁路0；15处正常滚动；最低普通文字对比度Light4.5327/Dark6.8625。修复Steps SLOT布局、220处指标说明对比度、9处状态文案。

2026-10-02恢复只读复查再次匹配99个稳定根、50状态、48弹窗、5,424实例、937导航和144关闭动作，失效主组件/目标0，四条入站目标正确，S13未创建。没有重新制作S12，也不把本次读回计为新的全量视觉/运行时验收。

## S11历史验收与边界

当前后端13项检查包含固定的external_model_runtime Warning。旧页面fixture文案不表示真实执行模型；设计没有伪造全报告Pass或部署按钮。报告生成、持久化、发布许可与部署分开。

A/B/C分别为11/2/0、9/1/3、9/4/0的通过/警告/阻断合计，各13项。40个专属证据弹窗含A/B/C各13项及独立D成本项；3个完整JSON示例保持report_id和合计一致。DEMO编号、数值和占位哈希不是真实后端身份或完整性证明。

请求窗口1–365天整数；预算可空或有限非负。null、0和超限独立，最新用量日期不一定是今天。无P95不补0ms；未运行不记0阻断。新请求等待期间保留的A标为上次报告，B加载/失败不展示A。历史只读，无覆盖或删除；结果未知先核验，不自动重发。

S11原验收为676条NAVIGATE/OVERLAY、159项CLOSE、0其他动作；33状态、53弹窗及2条S10入站。两个最终运行确认未执行；9,941节点/3,796实例、196禁用控件及22处正常滚动等记录继续保留于[S11独立状态](FIGMA_SCREEN_RELEASE_STATE.json)。S12增加S11到分析页的两条导航，不回写为S11此前已完成全产品集成。

## 页面与实现边界

始终使用现有组件真实实例，不detach、不创建重复Screen主组件、不用截图或旧QA代替产品目标。S12只修改自有节点及4条旧页面原型入站，无旧页面视觉/共享资产/应用源码变更；本次中断恢复只补齐记录。

任意分区、输入、任务控制器及取消/重试、H监控、事实修复、回包文件身份/幂等、审核说明、能力历史事务、私有提供器、影子样本、晋升/回滚事务及全部权限/主题/滚动/来源/焦点/草稿持续性尚未实现。部分刷新、诊断和字段入口提供规则说明，不等于完整控制器。所有数据及结果是预置示例。

保留先前边界：S11报告/测量/账本/发布分开；S10四项源码差异仍未修复，规则包读取/注册/绑定/执行独立；S09类别/文件/模式/批次及结果未知保护；S08预设与比赛副本及实时can_apply；S07来源身份、长期能力与短期标签；S06归档/删除/强清与导入/P4资格；S05隐藏不删血缘；S04查看/执行与复盘/结算、SHA保护；Timeline核验/修订/取消；S03输入/提供器/输出；S02双方11首发和失败草稿；AI Chat只读上下文主动勾选、session/request隔离。

不能把有限原型记成17路由端到端或真实产品发布通过。S13制作前核对现有AI Chat Pattern和独立会话/请求语义，不把S12回包导入等同于聊天上传或SQL执行，也不擅自制作S14配置页。

## 恢复

读取最新Screens计划、总状态、当前页详细状态及实际Figma稳定根。S11历史总检查点为 `a1ede69ac02f7ab7ac73e73485a44cc4e97f7eab`；S10为 `aeeee7700b5665566729da826e6ab2b0a81b39ae`；S09为 `4627fecbe7d83ce377c36db82d63aec2bed34712`；其余历史见Screens计划。

本次发现README/S12独立accepted与总状态/两份计划不同步，已依据独立记录和实际画布进行同步。遇到中断不能只凭旧总索引回退，也不能只凭较新WIP认定完成。先核对日期、阶段、稳定根和验收范围。

旧截图、空handoff和旧“Screens未开始”不能覆盖核对后的记录；SLOT虚拟ID按稳定根、语义名称与真实组件恢复，不猜ID。Create State handoff未创建，GitHub明确状态为恢复依据。
