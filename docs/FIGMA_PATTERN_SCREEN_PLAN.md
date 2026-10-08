# Figma Patterns & Screen Plan

**设计依据：Figma决定尺寸、比例、间距、字号和颜色；源码只提供功能、交互与数据语义。旧CSS不是新视觉依据。** 唯一设计记录分支`ui-design-system`。

## 当前状态

图标、底层组件与Patterns六组全部完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**2026-10-08，Screens已有15 / 17条路由通过可编辑视觉、代表性本页状态及有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams、S07 players、S08 lineup_presets、S09 workbooks、S10 rules、S11 release、S12 analytics、S13 api_workspace、S14 openai、S15 database。S16/S17尚未开始。** 累计518个产品状态仍只计15条路由；弹窗、Review Hub、上下文菜单和Pattern QA不增加路由数。真实运行时及完整跨路由端到端集成未完成。

恢复：[总状态](FIGMA_SCREENS_STATE.json) → [S15详细状态](FIGMA_SCREEN_DATABASE_STATE.json) → [S15独立记录](FIGMA_SCREEN_DATABASE.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S16 `logs` / 问题日志，尚未开始。**

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
- [x] 独立状态中明确列出的部分真实入站与返回。
- [ ] **S16 `logs`：下一项，尚未开始。**
- [ ] S17 `architecture`：尚未开始。
- [ ] 其余真实目标、全状态持续性、完整跨路由和运行时联调。

各页历史计数、节点、源码差异及运行时限制以独立记录为准。S15新增32状态/16弹窗/1 Review Hub，只增加一条产品路由。

## S15验收与边界

连接并保存会执行自动迁移与资料注册，不是只读测试。配置草稿B与活动A、服务器迁移与本机凭据保存分别判断；失败不保证整体回滚。清除连接不删除业务数据。未知健康和数量不补0，估算显示“约”，旧统计刷新时保留并标记。

重置作用于已保存目标，空/错/匹配/目标变化分别显示。名称相同不能保证主机与连接版本相同。schema删除事务先提交，再执行迁移；后续重建或重连失败不能宣称旧数据已恢复。CASCADE依赖影响和客户端无备份/撤销/恢复能力明确说明。数据库本身、本机配置保留，内置注册不等于所有表为0。

最终 **135 NAVIGATE、393 OVERLAY（合计528）、9 SWAP、32 CLOSE、其他0**。9 SWAP仅用于人工选择名称确认示例。Hub `676:187457`可达32状态和16弹窗。4条S01/S13/S14真实入站另计，无数据库入口进入未配置页，不误入健康示例。

3个最终操作确认和3个禁用重置门禁的最终按钮均无执行连接。没有实际连接、迁移、凭据写入、清除、备份或重置；结果从明确演示入口独立选择。异常、忙、结果未知和已处理状态不会经普通tab导航悄悄恢复旧A。验收目录是显式的预置退出入口。

6935未隐藏节点、2744嵌套实例、1732文字层完成检查。越界、失效引用/目标、未绑定颜色、顶层重叠、异常按钮和门禁错误均0；186禁用控件反应与祖先旁路0。1678普通文字层最低对比度Light4.5048/Dark6.8574，72处本地修正。动作栏位于viewport外；本组没有实际超高内容区域，不声称完成长内容压力测试。

## 页面与实现边界

始终复用真实实例，不detach、不创建重复Screen主组件、不用截图或旧QA代替产品目标。S15只修改本页和4条选定旧页原型入站；没有旧页视觉、共享资产或应用源码变更。

S15任意输入、真实PostgreSQL/DPAPI、连接与凭据失败恢复、完整目标版本绑定、任务与外部客户端并发、权限及外部备份、URL和日志全路径脱敏、真实迁移/重置结果核验、所有主题/滚动/焦点/草稿持续性仍需实现。设计保护不是运行时修复。

继续保留：S14选中/当前/密钥/测试独立及旧响应失效；S13上下文主动勾选和session/request/草稿隔离；S12精确分区许可及候选/历史/正式参数分离；S11报告完成不等于发布；S10读取/校验/注册/绑定/执行独立；S09文件/模式/批次隔离；S08预设与比赛草稿副本；S07长期能力/短期标签与来源身份；S06归档/删除/强清与导入/P4门槛；S05隐藏不删血缘；S04查看/执行与复盘/结算；Timeline核验/修订/取消；S03输入/提供器/输出；S02双方11首发与失败草稿。

上述既有独立记录和实现待办不因S15验收而消失。有限原型不是17路由端到端或真实产品发布通过。

## 恢复

读取最新Screens计划、总状态、S15详细状态、独立记录和实际Figma稳定根。S14同步历史总检查点为`52236b78b5a0f478bb6fb3b4235d30fd92d4e97c`；S13为`9ae16eaf438d3a8963fd2868e9b26611fd963dd4`；S12为`0cdcbf5fd4c878f7cb4a8995786677f8ea251263`；其余历史链见Screens计划。

早期WIP、旧截图、空handoff和旧“Screens未开始”不能覆盖新accepted。SLOT虚拟ID按稳定根、语义名称和真实主组件恢复，不猜编号。Create State handoff未创建；以GitHub独立记录和同步总检查点恢复。
