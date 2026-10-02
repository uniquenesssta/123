# Figma Patterns & Screen Plan

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互和数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支 `ui-design-system`。

## 当前状态

图标、底层组件与Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**2026-10-02，Screens已有13 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks、S10 rules、S11 release、S12 analytics、S13 api_workspace。S14–S17四条尚未开始。** 累计448个产品状态仍只计13条路由；弹窗、Review Hub、上下文菜单和Pattern QA不增加路由数量。真实运行时与完整跨路由端到端集成未完成。

恢复：[总状态](FIGMA_SCREENS_STATE.json) → [S13详细状态](FIGMA_SCREEN_API_WORKSPACE_STATE.json) → [S13独立记录](FIGMA_SCREEN_API_WORKSPACE.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S14 `openai` / 兼容API配置，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件与Composer。

各组独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat历史状态](FIGMA_AI_CHAT_STATE.json)与[Workflow历史状态](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)不能覆盖后续Screens进度。组件API未改变时，不为更新进度重写历史组件记录。

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
- [x] S12 `analytics`：历史、后台任务、质量/审核、回包、H/I与受控参数生命周期。
- [x] S13 `api_workspace`：普通文本问答、历史与上下文、session/request/草稿隔离、归档与历史审计。
- [x] 各独立状态明确记录的部分真实入站与返回。
- [ ] **S14 `openai`：下一项，尚未开始。**
- [ ] S15–S17：见独立Screens计划，尚未开始。
- [ ] 其余真实目标、全状态持续性、完整跨路由及运行时联调。

S13新增50状态、24弹窗与Review Hub `635:166265`，仍只增加一条产品路由。S01–S12历史计数和范围在独立记录保留；S12曾中断后补齐总索引，不重建页面。

## S13验收与边界

真实复用AI Chat Workspace、Sidebar/History Item、Message、Composer、Attachment及Shell/表单/Dialog。没有创建新的Screen主组件、detach或修改共享资产及应用源码。

客户端保存会话/消息账本与AI修改业务资料分开；只保留普通文本与主动勾选的只读上下文。TEAM-A、PLAYER-A01、MATCH-DEMO-01互不混用；既有会话锁定原API、类型与对象，缺少原密钥不自动重绑定。

REQ-A-01只属于A；A发送时浏览B，B正文不出现A待处理问题、B草稿不变、全局发送锁不能绕过。A完成仍留在B，返回A查看4条消息的新版本。B读取失败不借用A，未选择历史不自动选择A。NEW临时请求未确认前不算已保存的D。

取消待确认与终态分开；失败保留最近确认历史与草稿，不据此断言服务端没有保存新消息。结果未知先核验，不自动重发。归档只移出active历史，保留消息与审计，不删除业务资料；重置不是归档或取消。历史C的文件、HTML/SQL和提案仅普通文本与索引，不恢复执行能力。

最终 **1196条NAVIGATE/OVERLAY、72项CLOSE、0其他动作**，Hub可达50状态与24弹窗。S01/S12默认Light/Dark的4条真实AI入站另计，只保留代表性主题，不转移比赛、分析、文件、草稿或许可。

4个归档最终确认无执行连接；普通发送只进入模拟等待，后续结果由明确标记的人工演示入口选择。未发起真实API、数据库、凭据、剪贴板、上传或文件生成操作。

11615个未隐藏节点、4682嵌套实例、2658普通文字层检查通过；非预期越界、失效引用/目标、未绑定solid颜色、顶层重叠、异常按钮高度、会话断言错误均0。384禁用控件自身点击反应及父级旁路0；4处正常滚动；最低文字对比度Light4.5327/Dark6.8625。

修正历史tab双高亮、未连接状态、96处局部文字对比度和紧凑布局。长消息1564高在320视口内滚动，输入区在外；紧凑使用实例级布局，不声称自动断点。传输中断后已回读实际接线，不重复创建。

## 页面与实现边界

可编辑视觉、有限原型、完整跨路由和真实运行时分别验收。任意输入、全量会话、所有上下文发送组合、网络任务、取消/重试与落库核验、幂等、完整转义/权限/剪贴板、全部主题/折叠/滚动/来源/焦点/草稿持续性尚未实现。部分控件提供范围说明，不代表完整运行控制器。Escape是退出验收，不是取消真实请求。

保留此前边界：S12全局快照非精确分区、能力建议接受非能力历史写入、绑定改变禁旧候选晋升；S11报告完成非发布且公开运行时Warning保留；S10四项源码差异及读取/校验/注册/绑定/执行独立；S09文件/模式/批次与未知保护；S08预设/比赛副本独立；S07来源身份、长期能力与短期标签分开；S06归档/删除/强清及导入/P4独立；S05隐藏不删血缘；S04查看/执行、复盘/结算与SHA保护；S03输入/提供器/输出独立；S02双方11首发和失败草稿。

所有数据为预置；不把有限原型记成17路由端到端通过。S14兼容API、S15数据库当前提供说明，不用旧Pattern QA冒充目标。

## 恢复

读取最新Screens计划、总状态、当前页独立状态与实际Figma稳定根。S12历史总检查点 `0cdcbf5fd4c878f7cb4a8995786677f8ea251263`；S11 `a1ede69ac02f7ab7ac73e73485a44cc4e97f7eab`；S10 `aeeee7700b5665566729da826e6ab2b0a81b39ae`；更早链见Screens计划。

初始/组合WIP由accepted替代；旧截图、空handoff和旧“Screens未开始”不能覆盖最新记录。SLOT虚拟ID按稳定根、语义名称和真实主组件解析，不猜ID。Create State handoff未创建，GitHub明确检查点为恢复依据。下一项仅S14，不擅自推进S15–S17。
