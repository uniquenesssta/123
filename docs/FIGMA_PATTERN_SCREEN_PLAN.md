# Figma Patterns & Screen Plan

**设计依据（2026-08-28确认）：Figma决定视觉尺寸、比例、间距、字号和颜色；应用源码只提供功能、交互和数据语义。旧源码尺寸是历史取证，不是新设计依据。**

## 当前状态

截至 **2026-09-30**，图标、既定底层组件和Patterns六组均完成。Controls **61 sets /506 variants**；Patterns **17 sets /125 variants /13 production singles**；Foundations **193 variables /13 text styles /6 effects**。

**Screens已有5 / 17条路由完成可编辑视觉、代表性本页状态与有限原型验收：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs。S06–S17共12条未开始。** 132个累计产品状态画面仍只计5条路由；弹窗、上下文菜单、Review Hub和Pattern QA不增加路由数。

当前恢复入口：[Screens总状态](FIGMA_SCREENS_STATE.json) → [S05详细状态](FIGMA_SCREEN_RUNS_STATE.json) → [S05记录](FIGMA_SCREEN_RUNS.md)。执行顺序见[17路由计划](FIGMA_SCREENS_PLAN.md)。**下一项：S06 `teams` / 球队，尚未开始。**

## Patterns执行记录

- [x] PC App Shell：一级导航、二级导航、Topbar、折叠与主题保持。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector工作区（含Entity Row）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件、Composer。

依赖按组归档，不合并为大文档。证据：[Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)、[Master–Detail–Inspector](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow / Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat检查点](FIGMA_AI_CHAT_STATE.json)和[Workflow检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)是历史记录，不能覆盖后续Screens进度。

## 当前阶段：Screens

- [x] S01 `dashboard`：数据总览、连接前置及本页状态。
- [x] S02 `lineups`：比赛、双方阵容、模型链路、历史、页内工作包。
- [x] S03 `prediction`：正式推演、P4研究与收敛、最近历史、临时演练。
- [x] S04 `review`：九步复盘、手动补录、结果事件、候选与最近复盘的代表性状态。
- [x] S05 `runs`：独立运行列表、详情、技术追踪、列表隐藏及数据库前置。
- [x] 选定S01→S02、S01/S02→S03、S01/S02/S03→S04、S02/S03/S04→S05入口与明确产品返回。
- [ ] **S06 `teams`：下一项，尚未开始。**
- [ ] S07–S17：见独立Screens计划，均未开始。
- [ ] 其余真实目标、全状态持续性、完整跨路由和运行时联调。

历史S01为15状态，S02为33状态/7弹窗，S03为33状态/3弹窗，S04为33状态/18弹窗（含9只读步骤说明）。计数对应各次验收快照，后续增量由最新总状态记录，不重写未变化的组件API。

S05有18状态、14弹窗、1上下文菜单和1Review Hub `521:55004`，全部可达。216条NAVIGATE/OVERLAY、41项CLOSE和172项其他组件/继承动作分别统计；6条外部入站另计。1,944个未隐藏实例可解析；非预期越界、未绑定solid颜色、顶层重叠、异常按钮高度均0；5处正常滚动；127个Disabled无反应。普通文字最低对比度Light4.5048/Dark6.5682。

S05六条入站来自S02/S03/S04默认Light/Dark的Page/runs，分别进入S05 Light/Dark列表。其余身份传递、所有主题/折叠/刷新持续性没有因此完成。S04原六条入站仍到未选择比赛，不改变其原约束。

## 页面与实现边界

页面使用已完成组件真实实例；不detach、不创建重复Screen主组件、不将旧QA作为产品目标。S05没有修改应用源码、全局变量或既有主组件。

S05移除只隐藏列表项，保留运行、概率矩阵、快照、复盘与收敛血缘。缺失值不补为0或当前模型，历史输入审计不授予新执行许可。六条记录的查看/移除目标保持对象身份；刷新中和失败保留六行，独立隐藏后示例五行B–F。

八个Hide相关最终确认没有执行反应；提交中、失败和隐藏后状态由审查目录独立选择。24行压力场景的重复行操作禁用；上下文菜单可检查，但真实右键未实现。任意记录载荷、所有刷新/数据库/显隐/复位和完整主题/滚动持续性不声明为完成。

S04查看与allowed_actions分离；第4步Blocked时回看第2/8步不授予执行。SHA变化使旧确认失效，预检/确认/写事实/生成复盘/结算分别校验；候选接受才允许后端回写。Timeline修订不升级核验，取消不累计有效比分。手动合法任意提交和全部补充量化字段仍待实现。

S03输入门禁、独立ModelProvider和结果返回分开，公开源码不捆绑P4/P7算法；研究决策只追加、截止后只读、未知不等于0、冻结不可变。S02保持双方各11名首发、失败保留草稿、保存与准入独立及先预检后确认。

AI Chat保持纯文本、只读上下文主动勾选、Pending/取消绑定sessionId/requestId，不扩展上传、文件生成或数据库执行。

所有身份、比分、概率、任务、文件、结算和候选均为预置示例。真实API、数据库、SHA、文件、权限、模型、Worker、输入、键盘焦点、竞态及完整主题/草稿/滚动持续性须后续测试。有限原型不声称所有控件可交互或17路由端到端通过。

## 恢复

读取最新Screens计划、总状态、当前页详细状态和实际Figma稳定根。S04历史总状态在 `1d2b9b78d4d7bf2e589c81e69e4fef89638c98cb`；S03在 `ff60437ccd38e1ed169cbebad0b21f7d8610e128`。S05早期分配WIP已被accepted状态替代；批量工具连接错误后已读回确认18个根完成，不重复构建。

组件总记录中的Screens未开始、旧截图、空handoff及旧计数不得覆盖新检查点。组件API不为同步进度而重写；规范化SLOT虚拟ID按稳定根、语义名称及真实引用恢复，不猜ID。
