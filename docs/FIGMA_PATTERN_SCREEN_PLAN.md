# Figma Patterns & Screen Plan

**设计依据（2026-08-28确认）：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互和数据语义。旧CSS几何不是新设计依据。**

## 当前状态

截至 **2026-10-01**，图标、底层组件和Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有8 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets。S09–S17共9条未开始。** 累计250个产品状态仍只计8条路由，弹窗、上下文菜单、Review Hub、Pattern QA不增加路由数量。

恢复：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S08详细状态](FIGMA_SCREEN_LINEUP_PRESETS_STATE.json) → [S08记录](FIGMA_SCREEN_LINEUP_PRESETS.md)。顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S09 `workbooks` / Excel工作包，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件与Composer。

各组独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat历史状态](FIGMA_AI_CHAT_STATE.json)和[Workflow历史状态](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)不能覆盖后续Screens进度。

## 当前阶段：Screens

- [x] S01 `dashboard`：总览、连接前置及代表性状态。
- [x] S02 `lineups`：比赛、双方阵容、模型链路、历史、局部工作包。
- [x] S03 `prediction`：正式推演、研究、历史、临时演练。
- [x] S04 `review`：九步复盘、补录、事件和候选。
- [x] S05 `runs`：独立运行列表、详情、技术追踪、历史隐藏。
- [x] S06 `teams`：目录、名单、档案、阵型/任期、资料包。
- [x] S07 `players`：目录、来源身份、档案、有效履历、状态/能力/标签、工作包。
- [x] S08 `lineup_presets`：按球队管理预设、编辑校验、复制/归档/删除、套用边界。
- [x] 各独立页状态记录中列明的部分真实入站和返回连接。
- [ ] **S09 `workbooks`：下一项，尚未开始。**
- [ ] S10–S17：见独立Screens计划，尚未开始。
- [ ] 其他真实目标、全状态持续性、完整跨路由及运行时联调。

S01–S07历史计数与验收含义不重写。S08新增31状态、20弹窗和Review Hub `573:103157`；仍只增加一条产品路由。

S08最终回读 **516条NAVIGATE/OVERLAY、60项CLOSE、0其他动作**。31状态与20弹窗均可达，7条外部入站另计。5,139个未隐藏实例可解析；非预期越界、失效引用/目标、未绑定solid颜色、异常按钮高度和顶层重叠均0。249个禁用/忙控件及可点击父容器旁路均检查通过；30处正常滚动。普通文字最低对比度Light4.5048/Dark4.5490。

S06/S07通用入口进入对应主题的未选球队状态，不凭空带入A。S02主/客管理分别进入A/B对应球队，S06旧六人预设摘要只进入本队管理，不静默升级成A-P1十一首发。完整比赛草稿/返回/主题持续性没有因此完成。

## 页面与实现边界

使用现有组件真实实例，不detach、不创建重复Screen主组件、不用旧QA替代产品目标。S08没有修改源码、共享变量/样式/主组件，只修改自有节点和7条相关入站，并调整一处S06入口标签。

预设编辑验证名称、11首发、位置唯一与阵型槽位完全匹配、概率0–1或空；教练可选。动态编辑可以暂时非法，但保存禁用。默认方案在同一球队内唯一，角色继承与覆盖分开。

复制需新名称；归档保留历史且退出快速套用；永久删除可针对活动/归档预设，仅删除预设与成员关系，不影响球队、球员及已保存比赛副本。套用需最新can_apply及同队身份，只覆盖目标一侧草稿。11个最终操作确认没有执行连接，成功/错误场景独立选择。

完整编辑以A-P1为代表；其他预设只读摘要保持其身份。任意人员选择、真实表单校验、自动分配、全部预设载荷、实时伤停预检、版本冲突和数据库事务尚未实现；不把条件原型当作真实模型或比赛操作。

保留此前规则：S07独立目录不沿用来源球队，未知年龄/能力不补0，短期标签不覆盖长期能力；S06归档/删除/强制清除和导入/P4评分独立；S05隐藏不删血缘；S04查看/执行分离、SHA变化拒绝旧确认、复盘/结算分开；Timeline修订不升级核验、取消不累计比分；S03输入/提供器/输出独立；S02双方各11首发且失败保留草稿；AI Chat只读上下文主动勾选和session/request隔离。

所有实体、版本、文件、任务和结果均为预置示例。真实API/数据库/文件/SHA、权限、模型/Worker、键盘焦点、竞态，以及完整主题/折叠/滚动/草稿持续性须后续测试。有限原型不声明所有控件可执行或17路由端到端通过。

## 恢复

读取最新Screens计划、总状态、当前页详细状态与实际Figma稳定根。S07历史总状态为 `66d575f0d9985012a43448e2fed2af9f913c3117`；S06为 `ac85c1b42be55bd7378b0990a3406b6da4d9d087`。S08初始和组合WIP已被accepted替代；连接中断后已读回确认，不重复创建。

旧截图、空handoff、组件阶段“Screens未开始”不能覆盖新检查点。未变更的组件API不为更新进度而重写；SLOT虚拟ID按稳定根、语义名称和真实实例引用恢复，不猜ID。
