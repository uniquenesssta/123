# Figma Patterns & Screen Plan

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互与数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支 `ui-design-system`。

## 当前状态

图标、底层组件与Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**2026-10-08，Screens已有14 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks、S10 rules、S11 release、S12 analytics、S13 api_workspace、S14 openai。S15–S17三条尚未开始。** 累计486个产品状态仍只计14条路由；弹窗、Review Hub、上下文菜单与Pattern QA不增加路由数。真实运行时及完整跨路由端到端集成未完成。

恢复：[总状态](FIGMA_SCREENS_STATE.json) → [S14详细状态](FIGMA_SCREEN_OPENAI_STATE.json) → [S14独立记录](FIGMA_SCREEN_OPENAI.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S15 `database` / 数据库，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件与Composer。

独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

组件与Pattern旧检查点不覆盖后续Screens。共享API未改，不为进度更新重写历史组件记录。

## 当前阶段：Screens

- [x] S01 `dashboard`：总览、连接前置与代表性状态。
- [x] S02 `lineups`：比赛、双方阵容、链路、历史、局部工作包。
- [x] S03 `prediction`：正式推演、研究、历史、临时演练。
- [x] S04 `review`：九步复盘、补录、事件、候选。
- [x] S05 `runs`：运行列表、详情、技术追踪、历史隐藏。
- [x] S06 `teams`：目录、阵容、档案、任期/阵型、资料包。
- [x] S07 `players`：目录、来源身份、档案、履历、能力/标签、工作包。
- [x] S08 `lineup_presets`：预设编辑、校验、复制/归档/删除与套用边界。
- [x] S09 `workbooks`：三类工作包、导出/预检/提交、类别/文件/模式/批次隔离。
- [x] S10 `rules`：赛事目录、层级、模型路由、规则包。
- [x] S11 `release`：验收请求、分类检查、证据、性能/安全/成本、历史。
- [x] S12 `analytics`：历史、后台任务、质量/审核、回包、H/I及受控参数生命周期。
- [x] S13 `api_workspace`：普通文本问答、历史/上下文、session/request/草稿隔离及归档。
- [x] S14 `openai`：配置、密钥、示例解析、模型参数、测试与安全。
- [x] 各独立状态明确列出的部分真实入站与返回。
- [ ] **S15 `database`：下一项，尚未开始。**
- [ ] S16 `logs`、S17 `architecture`：尚未开始。
- [ ] 其余真实入口、完整跨路由、全状态持续性与运行时联调。

S14新增38状态/24弹窗与Review Hub `656:178986`，只增加一条产品路由。S01–S13的计数、历史验收和未实现范围继续保存在各自记录。

## S14验收与边界

复用现有Shell、Panel、Page Heading、Button、Text/Number/Select/Textarea及Dialog；没有新增Screen主组件、detach、共享变量/样式或应用源码修改。

选中编辑、当前启用、密钥存在、参数已保存、最小请求成功与正式研究能力分开。源码允许有密钥但未测试的B激活；Responses仅表示研究候选。解析curl/JSON不执行命令，不把YOUR_API_KEY当成有效新密钥；解析变更只影响草稿。

TEST-DEMO-A只属于API-A/r1；B不显示A的耗时/错误，当前r2不能被旧r1响应验证。保存后探测失败不撤销已保存参数；结果未知先核验。密钥不回显，留空/替换/移除不同。移除可保留当前标记但须禁用测试；删除不改绑旧会话。没有业务数据库仍可管理本机设置。

最终 **1065条NAVIGATE/OVERLAY、72项CLOSE、0其他动作**，Hub可达38状态和24弹窗。新增S13新会话Light/Dark侧栏与工具栏4条入站，另计，不转移会话、草稿、密钥或执行许可。

12最终操作确认均无执行连接；没有实际凭据保存、收费探测、激活、移除、删除或数据库操作。结果由显式“演示”按钮或Hub手动选择，不冒充网络返回。

8894未隐藏节点、3465实例、2399文字层完成深层检查。非预期越界、失效引用/目标、未绑定solid颜色、顶层重叠、异常按钮高度和状态断言错误均0；256禁用控件自身反应和父级旁路0；9处正常滚动。2278普通文字层测量，最低对比度Light4.5048/Dark7.0938，无未解析项。

修正Chat协议说明、测试摘要/侧栏一致性、已处理结果导航与同帧跳转；破坏性确认复用Danger语义。长参数独立滚动且保存栏在外；紧凑场景不意味着自动断点已实现。传输中断后回读实际画布，不重复生成。

## 实现范围与历史约束

可编辑视觉、有限原型、完整跨路由及真实运行时分别验收。S14任意输入/配置组合、真正的保存与凭据回滚、网络探测、异步解析归属、临时密钥清理、日志脱敏、计费/模型能力、旧会话引用及全部主题/滚动/焦点/草稿持续性尚待实现。当前update_test_state按profile_id回写，设计的测试连接版本重检仍需落实。

此前约束保留：S13消息账本不是AI业务写入，主动勾选只读上下文及A/B/NEW隔离；S12全局快照不是精确分区许可，接受建议不等于能力历史写入，绑定变化阻止旧候选晋升；S11报告完成非发布，保留公开运行时Warning；S10四项源码差异及读取/校验/注册/绑定/执行独立；S09文件/模式/批次/未知保护；S08预设/比赛副本独立；S07来源身份和长期/短期效果分开；S06归档/删除/强清及导入/P4独立；S05隐藏不删血缘；S04查看/执行、复盘/结算、SHA保护；S03输入/Provider/输出；S02双方11首发和失败草稿。

所有示例数据与结果均为预置，不把有限原型记录成17路由端到端通过。S14只修改自有节点及4处S13入站，未重新验收全部旧页。S15尚无真实产品目标，管理入口继续提供范围说明。

## 恢复

读取README、最新Screens计划、总状态、当前页独立状态和实际Figma稳定根。S13历史总检查点 `9ae16eaf438d3a8963fd2868e9b26611fd963dd4`；S12 `0cdcbf5fd4c878f7cb4a8995786677f8ea251263`；S11 `a1ede69ac02f7ab7ac73e73485a44cc4e97f7eab`；S10 `aeeee7700b5665566729da826e6ab2b0a81b39ae`；更早恢复链见Screens计划。

初始/组合WIP不能覆盖accepted，旧截图或空handoff不能替代明确记录。SLOT虚拟后代按稳定根、语义名称与真实主组件解析，不猜ID。Create State handoff未创建；使用GitHub独立记录和同步总检查点恢复。下一项仅S15，不擅自推进S16/S17。
