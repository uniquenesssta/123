# Figma Patterns & Screen Plan

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互和数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支 `ui-design-system`。

## 当前状态

截至 **2026-10-01**，图标、底层组件与Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有11 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks、S10 rules、S11 release。S12–S17六条尚未开始。** 累计348个产品状态仍只计11条路由；弹窗、Review Hub、上下文菜单和Pattern QA不增加路由数量。真实运行时与完整跨路由端到端集成未完成。

恢复：[总状态](FIGMA_SCREENS_STATE.json) → [S11详细状态](FIGMA_SCREEN_RELEASE_STATE.json) → [S11独立记录](FIGMA_SCREEN_RELEASE.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S12 `analytics` / 分析，尚未开始。**

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
- [x] 独立状态中明确列出的部分真实入站与返回。
- [ ] **S12 `analytics`：下一项，尚未开始。**
- [ ] S13–S17：见独立Screens计划，尚未开始。
- [ ] 其余真实目标、全状态持续性、完整跨路由与运行时联调。

历史S01–S10计数和验收含义保持于各自记录。S11新增33状态、53弹窗和Review Hub `599:136849`，只增加一条产品路由。

## S11验收与边界

当前后端13项检查包含固定的external_model_runtime Warning。旧页面fixture文案不表示真实执行模型；当前设计不伪造全报告Pass或部署按钮。报告生成、持久化、发布许可与部署分开。

A/B/C分别为11/2/0、9/1/3、9/4/0的通过/警告/阻断合计，各13项。40个专属证据弹窗含A/B/C各13项及独立D成本项；3个完整JSON示例保持report_id和合计一致。DEMO编号、数值和占位哈希不是真实后端身份或完整性证明。

请求窗口1–365天整数；预算可空或有限非负。null、0和超限独立，最新用量日期不一定是今天。无P95不补0ms；未运行不记0阻断。新请求等待期间保留的A标为上次报告，B加载/失败不展示A。历史只读，无覆盖或删除；结果未知先核验，不自动重发。

最终 **676条NAVIGATE/OVERLAY、159项CLOSE、0其他动作**。Hub可达33状态和53弹窗；S10默认Light/Dark的2条入站另计，只保留主题，不转移规则候选、比赛或许可。2个最终运行确认反应为0，不执行真实验收、账本写入、模型、成本接口或部署。

9,941个未隐藏节点、3,796嵌套实例、2,742普通文字层完成深层检查。非预期越界、失效主组件/目标、未绑定solid颜色、顶层重叠、异常按钮高度及证据身份错误0。196禁用/忙控件自身反应和父级旁路0；22处正常滚动；最低普通文字对比度Light4.5048、Dark6.8625。

158处本地文字对比度修正；62处继承的CHANGE_TO演示动作移除。长JSON使用原生滚动Frame包住完整Dialog Body实例，Header/关闭栏在滚动区外。未改共享主组件。

## 页面与实现边界

始终使用现有组件真实实例，不detach、不创建重复Screen主组件、不用截图或旧QA代替产品目标。S11只修改自有节点及2条S10原型入站，无旧页面视觉/共享资产/应用源码变更。

任意参数输入和报告组合、真实Windows/PostgreSQL测量、外部模型验证、凭据/成本读取、不可变账本事务与哈希核验、幂等和结果未知恢复、全部主题/折叠/焦点/滚动/草稿持续性尚未实现。部分刷新、类别和组合导航提供范围说明，不等于完整控制器；B/C未单独制作的类别显示自己的完整报告，不借用A。

保留先前边界：S10四项源码差异仍未修复，规则包读取/注册/绑定/执行独立；S09类别/文件/模式/批次及结果未知保护；S08预设与比赛副本及实时can_apply；S07来源身份、长期能力与短期标签；S06归档/删除/强清与导入/P4资格；S05隐藏不删血缘；S04查看/执行与复盘/结算、SHA保护；Timeline核验/修订/取消；S03输入/提供器/输出；S02双方11首发和失败草稿；AI Chat只读上下文主动勾选、session/request隔离。

所有数据和结果是预置示例；不能把有限原型记成17路由端到端或真实产品发布通过。

## 恢复

读取最新Screens计划、总状态、当前页详细状态及实际Figma稳定根。S10历史总检查点为 `aeeee7700b5665566729da826e6ab2b0a81b39ae`；S09为 `4627fecbe7d83ce377c36db82d63aec2bed34712`；其余历史见Screens计划。

S11初始/组合WIP由accepted覆盖，不重复创建已有根。旧截图、空handoff和旧“Screens未开始”不能覆盖新记录；SLOT虚拟ID按稳定根、语义名称与真实组件恢复，不猜ID。Create State handoff未创建，GitHub明确状态为恢复依据。
