# Figma Patterns & Screen Plan

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互与数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支`ui-design-system`。

## 当前状态

图标、底层组件与Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。S16沿用既有资产清单，没有修改共享资产。

**2026-10-08，Screens已有16 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks、S10 rules、S11 release、S12 analytics、S13 api_workspace、S14 openai、S15 database、S16 logs。只有S17 architecture尚未开始。** 累计548个产品状态仍只计16条路由；弹窗、Review Hub、上下文菜单和Pattern QA不增加路由数。真实运行时及完整跨路由端到端集成未完成。

恢复：[总状态](FIGMA_SCREENS_STATE.json) → [S16详细状态](FIGMA_SCREEN_LOGS_STATE.json) → [S16独立记录](FIGMA_SCREEN_LOGS.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S17 `architecture` / 架构信息，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级/二级导航、Topbar、折叠与主题。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件与Composer。

独立归档：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card/Panel](FIGMA_METRIC_ACTION_PANEL.md)、[MDI](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow/Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。组件和Pattern旧检查点不覆盖后续Screens；共享API未改，不为更新进度重写历史组件记录。

## 当前阶段：Screens

- [x] S01 `dashboard`：总览、连接前置与代表性状态。
- [x] S02 `lineups`：比赛、双方阵容、链路、历史与局部工作包。
- [x] S03 `prediction`：正式推演、研究、历史与临时演练。
- [x] S04 `review`：九步复盘、手动补录、事件与候选。
- [x] S05 `runs`：运行列表、详情、追踪与历史隐藏。
- [x] S06 `teams`：目录、阵容、档案、任期/阵型与资料包。
- [x] S07 `players`：目录、来源身份、档案、能力/状态/标签与工作包。
- [x] S08 `lineup_presets`：球队预设、编辑校验、活动/归档与套用边界。
- [x] S09 `workbooks`：三类工作包、导出/预检/提交与批次隔离。
- [x] S10 `rules`：赛事目录、层级、路由与规则包。
- [x] S11 `release`：验收请求、分类检查、报告证据、性能/安全/成本与历史。
- [x] S12 `analytics`：历史、任务、质量、审核、回包与参数生命周期。
- [x] S13 `api_workspace`：普通文本、会话、只读上下文、请求与归档审计。
- [x] S14 `openai`：配置、密钥、示例解析、模型参数、测试与安全。
- [x] S15 `database`：连接与自动迁移、健康统计、清除本机连接与危险重置。
- [x] S16 `logs`：历史问题、重复聚合、技术详情、快照导出和清空边界。
- [x] 独立状态中明确列出的部分真实入站与返回。
- [ ] **S17 `architecture`：下一项，尚未开始。**
- [ ] 其余真实目标、全状态持续性、完整跨路由和运行时联调。

各页历史计数、节点、源码差异及运行时限制以独立记录为准。S16新增30状态/12弹窗/1 Review Hub，只增加一条产品路由。

## S16验收与边界

问题历史不等于当前仍在异常，空日志不等于系统健康。独立问题、总发生、重复和critical问题条数分别计数；同一保留occurrence_key不重复增量，新尝试可以增加，保留上限不表示无限历史。

本机日志在无数据库时可读；连接后的刷新还可扫描失败任务。扫描失败与本机列表读取成功可以共存。未取得/读取失败不补0，刷新保留旧快照。报告仅为导出时刻的本机文本快照，不上传、不自动更新，也不是完整runtime trace。

清空内存先于落盘；失败不证明原状态仍然完整，未知结果不自动重试。清空不修复业务故障、不清业务库或独立运行日志；重新捕获同指纹从新的记录周期开始。集合变化禁用旧确认仍需运行时版本绑定。PostgreSQL口令定向脱敏不等于全面敏感信息审查。

最终 **150 NAVIGATE +336 OVERLAY =486，0 SWAP、36 CLOSE、其他0**。Hub `686:195370`可达30状态和12弹窗。4条S15入站另计：默认Light/Dark同主题，恢复/重建失败进入无数据库的本机日志，不虚构健康数据库或授权。

两个最终操作示意与一个集合变化禁用确认的最终按钮均无执行连接。没有实际采集、文件导出、日志清空或故障修复。导出保存是原生保存位置选择的有限示意，不额外增加强制业务确认；结果为明确人工预置，复杂终态和未知导航不能悄悄恢复默认集合。

7005可见节点、2562嵌套实例、2032文字层完成检查。越界、失效引用/目标、未绑定颜色、顶层重叠、异常按钮和计数门禁错误均0；145禁用控件反应与祖先旁路0。2007普通文字层最低对比度Light4.5048/Dark6.5682，20处局部修正。11处实际超高滚动，长技术文本及报告经过检查，未测试完整500条极限。报告关闭栏和页面工具栏保持在内容滚动区之外。

## 页面与实现边界

始终复用真实实例，不detach、不创建重复Screen主组件、不用截图或旧QA代替产品目标。S16只修改本页和4条选定S15原型入口；没有旧页视觉、共享资产或应用源码变更。

S16任意日志记录/状态组合、全部捕获入口、原生文件保存、文件权限与原子恢复、清空确认快照版本、存储损坏显式错误、全面token/路径/个人信息脱敏、最大集合压力和完整主题/滚动/焦点持续性仍需实现。设计保护不是运行时修复。

继续保留：S15连接含迁移、清除与重置独立、schema删除提交后重建失败不能恢复旧数据，目标版本及并发仍需验证；S14选中/当前/密钥/测试独立及旧响应失效；S13上下文主动勾选和session/request/草稿隔离；S12精确分区许可及候选/历史/正式参数分离；S11报告完成不等于发布；S10读取/校验/注册/绑定/执行独立；S09文件/模式/批次隔离；S08预设与比赛草稿副本；S07长期能力/短期标签与来源身份；S06归档/删除/强清与导入/P4门槛；S05隐藏不删血缘；S04查看/执行与复盘/结算；Timeline核验/修订/取消；S03输入/提供器/输出；S02双方11首发与失败草稿。

上述既有独立记录和实现待办不因S16验收而消失。S17静态架构信息与运行时健康也须区分。有限原型不是17路由端到端或真实产品发布通过。

## 恢复

读取最新Screens计划、总状态、S16详细状态、独立记录和实际Figma稳定根。S15同步总检查点为`920432210478b26e17f2a3c9ffaf74c0f9f01e6f`；S14为`52236b78b5a0f478bb6fb3b4235d30fd92d4e97c`；S13为`9ae16eaf438d3a8963fd2868e9b26611fd963dd4`；S12为`0cdcbf5fd4c878f7cb4a8995786677f8ea251263`；其余历史链见Screens计划。

早期WIP、旧截图、空handoff和旧“Screens未开始”不能覆盖新accepted。SLOT虚拟ID按稳定根、语义名称和真实主组件恢复，不猜编号。Create State handoff未创建；以GitHub独立记录和同步总检查点恢复。
