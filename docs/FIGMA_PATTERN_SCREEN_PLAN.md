# Figma Patterns & Screen Plan

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互和数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支 `ui-design-system`。

## 当前状态

截至 **2026-10-01**，图标、底层组件及Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有10 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks、S10 rules。S11–S17七条尚未开始。** 累计315个产品状态仍只计10条路由，弹窗、Review Hub、上下文菜单和Pattern QA不增加路由数量。完整跨路由和真实应用运行时未完成。

恢复：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S10详细状态](FIGMA_SCREEN_RULES_STATE.json) → [S10独立记录](FIGMA_SCREEN_RULES.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S11 `release` / 发布验收，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件与Composer。

各组独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat历史状态](FIGMA_AI_CHAT_STATE.json)和[Workflow历史状态](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)不能覆盖后续Screens进度。未改变组件API不为更新进度重写其历史记录。

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
- [x] 各独立状态记录中明确列出的部分真实入站与返回。
- [ ] **S11 `release`：下一项，尚未开始。**
- [ ] S12–S17：见独立Screens计划，尚未开始。
- [ ] 其余真实目标、全状态持续性、完整跨路由和运行时联调。

S01–S09历史计数和验收含义保持于各自记录。S10新增35状态、27弹窗、Review Hub `591:124271`，只增加一条产品路由。

## S10验收与边界

最终 **697条NAVIGATE/OVERLAY、81项CLOSE、0其他动作**。Hub可达35状态和27弹窗，6条外部入站另计：S01/S03/S09默认Light/Dark模型导航进入真实S10，仅保留代表性主题，不转移比赛、文件、绑定或授权。

3,559个未隐藏嵌套实例可解析，8,939个节点完成深层检查。非预期越界、失效引用/目标、未绑定solid颜色、顶层重叠和异常按钮高度均0；205个禁用/忙控件及可点击父容器旁路通过；23处正常滚动。最低普通文字对比度Light4.5327/Dark6.8625。11个最终创建/停用/绑定/注册确认没有执行连接。

删除赛事实际为停用目录与赛事级绑定，保留历史比赛/推演/复盘；C1–C4身份与路径说明分开。目录级规则不等于本场路径或执行许可。上级C1改为C2后不沿用原S1/G1；空上级、无P4包、忙或结果未知禁保存。

规则包读取、后端校验、注册、绑定和执行资格独立。P7只读；读取中/无效/变化文件禁旧候选注册。注册中、结果未确认、已注册读回禁当前重复提交；成功读回从Hub独立选择，不由确认按钮伪造。

本轮明确记录四项源码差异：绑定表单可选与处理器必选赛事不一致，目录与路径查询P4过滤不一致，倒序列表按key建Map可能保留旧包，以及浅JSON校验/候选失效保护。没有修改源码，也不据此断言后端全无保护。

## 页面与实现边界

始终使用现有组件真实实例，不detach、不创建重复整页主组件、不用截图或旧QA代替产品目标。S10仅修改自有节点和6条旧页面原型入站，无旧页面视觉/共享主组件/变量/样式/应用源码变更。

任意筛选、表单和对象版本、全部父级切换、日期与重复校验、真实文件读取/schema兼容性、创建/停用/绑定/注册事务与幂等、完整操作控制器、所有主题/折叠/滚动/草稿/候选/焦点持续性尚未实现。部分刷新、格式、版本与核验入口展示规范而非执行逻辑；局部禁用状态不证明全部跨页竞态正确。

保留先前边界：S09类别/文件/模式/批次独立，无变更/已导入/失效/结果未知禁提交；S08预设与比赛副本独立、实时can_apply和同队校验；S07未知年龄/能力不补0、来源身份和长期能力/短期标签分离；S06归档/删除/强清和导入/P4资格独立；S05隐藏不删血缘；S04查看/执行、复盘/结算分开且SHA变化拒绝旧确认；Timeline核验/修订/取消独立；S03输入/提供器/输出独立；S02双方各11首发且失败保留草稿；AI Chat只读上下文主动选择和session/request隔离。

所有赛事、球员、文件、版本、任务和结果均为预置示例。真实API/数据库、权限、模型/Worker、键盘焦点及异步恢复需实现阶段验证；不把有限原型当作17路由端到端通过。

## 恢复

读取最新Screens计划、总状态、当前页详细状态及实际Figma稳定根。S09历史总检查点为 `4627fecbe7d83ce377c36db82d63aec2bed34712`；S08为 `06eee1a492a92b620485d194cdd83844ecadef13`；其余历史见Screens计划。

S10初始/组合WIP已被accepted覆盖，不重复创建已有根。旧截图、空handoff和旧“Screens未开始”不能覆盖最新记录；SLOT虚拟ID按稳定根、语义名称和真实主组件恢复，不猜ID。
