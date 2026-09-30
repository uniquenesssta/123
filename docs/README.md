# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并成一份大记录。

**设计依据（2026-08-28 用户确认）：视觉尺寸、比例、间距、字号和颜色以已确认的Figma为准。应用源码只用于功能、交互和数据语义，不作为视觉尺寸参考；旧源码尺寸只作历史取证。**

## 当前恢复入口

**2026-09-30最新检查点：S05 runs / 运行记录的可编辑视觉、代表性本页状态及有限原型已验收。** Screens当前 **5 / 17 条产品路由**：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs；S06–S17共12条尚未开始。

- [Screens总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S05运行记录：列表、详情、隐藏与血缘边界](FIGMA_SCREEN_RUNS.md)
- [S05明确恢复状态与节点账本](FIGMA_SCREEN_RUNS_STATE.json)
- [S04赛后复盘：九步、事实、结算和候选边界](FIGMA_SCREEN_REVIEW.md)
- [S04独立恢复状态](FIGMA_SCREEN_REVIEW_STATE.json)
- [S03赛事推演：原验收记录](FIGMA_SCREEN_PREDICTION.md)
- [S03独立恢复状态](FIGMA_SCREEN_PREDICTION_STATE.json)
- [S02比赛与阵容：原验收记录](FIGMA_SCREEN_LINEUPS.md)
- [S02独立恢复状态](FIGMA_SCREEN_LINEUPS_STATE.json)
- [S01 Dashboard：原验收记录](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S06 `teams` / 球队，尚未开始。** S05包含18个状态、14弹窗、1上下文菜单和1Review Hub，仍只计一条产品路由；累计132个状态不等于132条路由。

## 组件与模式记录

- [组件阶段记录与Figma节点](FIGMA_COMPONENT_RECORD.md)
- [按钮、Spinner与Loading组件](FIGMA_BUTTON_COMPONENTS.md)
- [扩展字段组件](FIGMA_EXTENDED_FIELDS.md)
- [Searchable Combobox组件](FIGMA_SEARCHABLE_COMBOBOX.md)
- [Switch / Toggle组件](FIGMA_SWITCH_TOGGLE.md)
- [Tabs / Segmented Control组件](FIGMA_TABS_SEGMENTED_CONTROL.md)
- [Data Table / Pagination组件](FIGMA_DATA_TABLE_PAGINATION.md)
- [Dropdown / Context / Overflow Menu组件](FIGMA_DROPDOWN_CONTEXT_OVERFLOW_MENU.md)
- [Dialog：普通、确认与名称校验危险确认](FIGMA_DIALOG.md)
- [Toast / Inline Alert / Blocking Message与Task Activity](FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md)
- [Accordion / Disclosure与展开状态恢复](FIGMA_ACCORDION_DISCLOSURE.md)
- [Progress / Skeleton / Empty State与异步状态边界](FIGMA_PROGRESS_SKELETON_EMPTY_STATE.md)
- [Avatar：球队、球员与默认占位](FIGMA_AVATAR.md)
- [PC App Shell：导航、顶栏与交互验收](FIGMA_PC_APP_SHELL.md)
- [Page Heading / Toolbar / Filter Bar / Selection Command Bar](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)
- [Metric Card / Action Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)
- [Master–Detail–Inspector与Entity Row](FIGMA_MASTER_DETAIL_INSPECTOR.md)
- [Workflow Stepper / Timeline与交互验收](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)
- [Workflow / Timeline历史检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)
- [AI Chat：历史、消息、只读附件与Composer](FIGMA_AI_CHAT.md)
- [AI Chat历史检查点与节点账本](FIGMA_AI_CHAT_STATE.json)
- [应用图标待办](FIGMA_ICON_BACKLOG.md)
- [基础Token与底层组件待办](FIGMA_FOUNDATION_COMPONENT_BACKLOG.md)
- [Patterns与Screens总阶段计划](FIGMA_PATTERN_SCREEN_PLAN.md)

执行顺序：图标 → 基础依赖与组件 → Patterns → Screens。

图标、既定底层组件及Patterns六组全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S01–S05使用现有真实实例，没有新建Screen主组件、detach或修改全局资产。

## S05验收摘要

独立运行列表、只读详情、技术追踪与历史隐藏确认已组合。模型、窗口、输入审计、指纹、比分和概率保留记录身份。缺失值不变成0，输入就绪不等于可重新运行；“移除”只隐藏列表项，不删除运行、矩阵、快照、复盘或收敛血缘。

Review Hub `521:55004` 可达18状态、14弹窗及上下文菜单。216条NAVIGATE/OVERLAY、41项CLOSE和172项其他组件/继承动作分别统计。6条S02/S03/S04默认Light/Dark入站另计，进入对应Light/Dark列表。

深层检查5,311个未隐藏节点、1,944个嵌套实例及1,532个普通文字层；实例均可解析。非预期越界、未绑定可见solid颜色、顶层重叠和异常按钮高度均0。127个Disabled控件无反应；5处正常垂直滚动单列；普通文字最低对比度Light **4.5048:1**、Dark **6.5682:1**。

首次加载数量“—”，成功空列表0；刷新中/失败保留六行；隐藏后独立示例只有B–F五行。A–F查看与确认目标逐条核对；24行压力场景的行操作禁用，不冒充24条独立记录。

修复只在本页实例：详情label/value宽度、紧凑2×2指标、52处低对比文字、14个继承Dialog事实容器及2处数据库显隐文案。批量写入的连接中断后已读回18根确认全部落盘，没有盲目重做或重复创建。

## 历史与实现边界

S01原验收15状态；S02为33状态/7弹窗；S03为33状态/3弹窗；S04为33状态/18弹窗（其中9个为只读步骤说明）。这些和S05的18状态累计132，只计5条路由。原未接线和计数对应当次快照，后续集成增量以最新状态为准。

S02保持双方各11名首发、保存与模型准入独立、失败保留草稿。S03保持输入门禁、ModelProvider与真实结果分开，未接入默认入口不跳到条件成功。S04保持步骤查看/执行权限分离、SHA变化拒绝旧确认、生成复盘与结算独立、取消事件保留审计、候选接受才回写。

S05八个Hide相关最终确认均没有写入反应；隐藏中、失败和隐藏后的画面从目录单独选择，不代表真实请求。右键事件、任意记录详情、实际数据库、全部控制器逻辑和完整主题/滚动/上下文持续性均未实现。部分按钮仅有语义与视觉，不声明所有控件可交互。

完整17路由集成、文件/SHA、数据库事务、模型/Worker、结算与能力回写、键盘焦点、异步竞态以及S04合法手动提交/补充字段等仍是后续范围。所有体育、模型、文件、时间、概率和请求结果为预置示例。

## 换会话恢复

先读本README、`FIGMA_SCREENS_PLAN.md`、`FIGMA_SCREENS_STATE.json`，再读 `FIGMA_SCREEN_RUNS_STATE.json`及S05记录，核对Figma稳定根。S05分配阶段WIP已被accepted检查点替代，不能按早期WIP回退重做。

S04旧总状态保留在 `1d2b9b78d4d7bf2e589c81e69e4fef89638c98cb`；S03在 `ff60437ccd38e1ed169cbebad0b21f7d8610e128`；S02在 `6aaff66b66fdcdc9f93f2fb74436b9534412f5cc`；S01在 `7a6891c571ffa07015774ac6b572c93b12c4c315`。各页独立记录继续保留。

不重写未变化的组件API。旧截图、空handoff、组件阶段Screens未开始文字不能覆盖最新明确检查点；规范化SLOT虚拟ID按稳定根、组件引用及语义路径恢复，不猜ID。
