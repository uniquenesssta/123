# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并成一份大记录。

**设计依据（2026-08-28用户确认）：视觉尺寸、比例、间距、字号和颜色以已确认的Figma为准。应用源码只用于功能、交互和数据语义，不作为视觉尺寸参考。**

## 当前恢复入口

**2026-09-30最新检查点：S06 teams / 球队的可编辑视觉、代表性本页状态与有限原型已验收。** Screens当前 **6 / 17条产品路由**：S01 dashboard、S02 lineups、S03 prediction、S04 review、S05 runs、S06 teams。S07–S17共11条尚未开始。

- [Screens总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S06球队：目录、档案、资料包与危险操作边界](FIGMA_SCREEN_TEAMS.md)
- [S06完整恢复状态与节点账本](FIGMA_SCREEN_TEAMS_STATE.json)
- [S05运行记录：原验收记录](FIGMA_SCREEN_RUNS.md)
- [S05独立恢复状态](FIGMA_SCREEN_RUNS_STATE.json)
- [S04赛后复盘：原验收记录](FIGMA_SCREEN_REVIEW.md)
- [S04独立恢复状态](FIGMA_SCREEN_REVIEW_STATE.json)
- [S03赛事推演：原验收记录](FIGMA_SCREEN_PREDICTION.md)
- [S03独立恢复状态](FIGMA_SCREEN_PREDICTION_STATE.json)
- [S02比赛与阵容：原验收记录](FIGMA_SCREEN_LINEUPS.md)
- [S02独立恢复状态](FIGMA_SCREEN_LINEUPS_STATE.json)
- [S01 Dashboard：原验收记录](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S07 `players` / 球员，尚未开始。** S06含42状态、27弹窗及1Review Hub，只计一条产品路由。S01–S06累计174个状态画面，不是174条路由；S06的球员速览、预设摘要、资料包不等于S07、S08、S09完成。

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

既有底层组件和Patterns六组全部完成。Controls **61 sets /506 variants**，Patterns **17 sets /125 variants /13 production singles**，Foundations **193 variables /13 text styles /6 effects**。S01–S06使用真实实例，没有创建Screen主组件、detach或修改共享资产。

## S06验收摘要

目录、筛选/游标分页、两项选择集合、A/B球队阵容、速览、完整档案、阵型观察、教练任期、历史/预设、完整资料包、新增资料及数据库前置已组合。A完整编辑表单为代表，B–F保持各自只读摘要；加载B时不沿用A名单。缺失资料不补成0或可用。

普通归档保留历史；普通删除仅针对无引用C，保护存在引用的A。强制清除独立按球队预检范围与后端 `preview.confirmation_text` 验证，不等于清空数据库。17个写相关最终确认反应数均0。

资料包无结构阻断且有待处理行才允许导入；65分可导入示例仍低于P4门槛70。86分但ready=0不可导入。解决一个身份冲突不清除其他格式错误；写入中禁重复提交，结果不明先核验，不自动重发。导入成功不等于模型可用或执行获准。

Review Hub `537:69771` 可达42状态/27弹窗，最后读回 **658条NAVIGATE/OVERLAY、77项CLOSE、0其他动作**。十条S01–S05默认Light/Dark资源主导航入站单独计数，进入对应主题目录，不伪装任意球队/草稿持续性。

深层11,662个未隐藏节点、4,723个真实嵌套实例、2,962个普通文字层；引用可解析。非预期越界、未绑定solid颜色、顶层重叠、异常按钮高度及失效目标0。256个禁用/忙控件无反应，可点击父容器旁路0；8处正常滚动。普通文字最低对比度Light **4.5048:1**、Dark **6.6589:1**。

修复本页弹窗/Hub的轴向及HUG/FILL尺寸依赖、选择说明高度、167处低对比文字、创建Panel误接点击、资料包计数及中文搜索/显隐文字。新增球队/教练的空名称禁用按钮及其父容器已重新核对，不能触发确认。

## 历史与实现边界

S01原15状态；S02原33状态/7弹窗；S03原33状态/3弹窗；S04原33状态/18弹窗；S05原18状态/14弹窗/上下文菜单。旧计数、旧未接线表述属于当次快照，后续集成增量以最新总状态为准，历史独立记录不回写为全产品已完成。

S02保持双方11名首发、保存/准入独立与失败保留草稿；S03保持输入门禁、ModelProvider和输出独立；S04保持查看/执行权限分离、SHA变化拒绝旧确认、复盘/结算分开；S05移除只隐藏列表、不删除运行血缘。

S06复选框任意选择、真实筛选与分页、所有球员速览、B–F完整编辑、任意表单输入、文件读写/导出、数据库事务、归档/删除/强制清除与全状态主题/草稿/滚动持续性尚未实现。17个最终写相关确认不连假成功；结果场景通过评审目录独立查看。部分控件保留视觉与语义，没有宣称全部可交互。

完整17路由集成、真实权限/竞态、模型/Worker、结算及能力回写、键盘焦点，以及先前页面保留的实现缺口仍需后续完成。所有球队、球员、日期、文件和统计均为预置示例。

## 换会话恢复

先读本README、`FIGMA_SCREENS_PLAN.md`、`FIGMA_SCREENS_STATE.json`，再读 `FIGMA_SCREEN_TEAMS_STATE.json` 与S06记录，核对实际Figma稳定根。S06分配/组合阶段WIP已被accepted替代，不回退重做。

S05旧总状态保留在 `444ad13d6e2de9fe9d99eb9a3c15022b40cd2ef4`；S04在 `1d2b9b78d4d7bf2e589c81e69e4fef89638c98cb`；S03在 `ff60437ccd38e1ed169cbebad0b21f7d8610e128`；S02在 `6aaff66b66fdcdc9f93f2fb74436b9534412f5cc`；S01在 `7a6891c571ffa07015774ac6b572c93b12c4c315`。

不要为更新进度重写未变化的组件API。旧截图、空handoff、组件阶段Screens未开始文字不能覆盖最新明确状态；规范化SLOT虚拟ID按稳定根、真实组件引用和语义路径恢复，不猜ID。
