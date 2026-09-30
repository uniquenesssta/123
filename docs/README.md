# Figma Design System Docs

按需读取对应文件，不把不同层级的信息合并成一份大记录。

**设计依据（2026-08-28 用户确认）：视觉尺寸、比例、间距、字号和颜色均以已确认的Figma为准。应用源码只用于功能入口、交互和数据语义，不作为视觉尺寸参考；旧源码尺寸只作历史取证。**

## 当前恢复入口

**2026-09-30最新检查点：S03 prediction / 赛事推演的视觉、本页适用状态及有限原型已验收。** Screens当前 **3 / 17 条产品路由**：S01 dashboard、S02 lineups、S03 prediction；S04–S17共14条尚未开始。

- [Screens总计划：17条路由与下一项](FIGMA_SCREENS_PLAN.md)
- [最新Screens总检查点](FIGMA_SCREENS_STATE.json)
- [S03赛事推演：设计、外部提供器边界、状态及验收](FIGMA_SCREEN_PREDICTION.md)
- [S03明确恢复状态与节点账本](FIGMA_SCREEN_PREDICTION_STATE.json)
- [S02比赛与阵容：原验收记录](FIGMA_SCREEN_LINEUPS.md)
- [S02独立恢复状态](FIGMA_SCREEN_LINEUPS_STATE.json)
- [S01 Dashboard：原验收记录](FIGMA_SCREEN_DASHBOARD.md)

**下一项：S04 `review` / 赛后复盘，尚未开始。** S03有33个状态、3个对话框、1个Review Hub，仍只计一条路由；其最近历史不代表S05运行记录页面完成。

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

图标、既定底层组件及Patterns六组全部完成。Controls保持 **61 sets /506 variants**，Patterns保持 **17 sets /125 variants /13 production singles**，Foundations保持 **193 variables /13 text styles /6 effects**。S01–S03使用现有真实实例，没有新建Screen主组件或修改全局资产。

## S03验收摘要

四个页内区域为正式推演、P4研究与收敛、最近历史、临时演练。**当前源码未捆绑外部ModelProvider**：输入就绪、提供器可用、结果返回是独立状态。条件结果显示预置概率，不代表实际计算；正常未接入入口不会直接跳到成功结果。

正式阻断与影子许可分开，窗口变化使旧许可/指纹失效；影子入口失败不开放正式运行。研究人工决策只追加，采用来源与接受未知是独立示例；截止后只读，冻结快照不可变。临时演练不保存比赛，历史隐藏不删除运行血缘。

Review Hub `498:32144` 可达全部33状态和3对话框。最终323条NAVIGATE/OVERLAY、9项CLOSE已核对，386项其他组件/继承动作另计。7条来自S01/S02的入站不计入323；S02原Blocked入口仍Disabled、反应数0。

深层检查8,510个未隐藏节点、3,339个未隐藏嵌套实例及2,057个普通文字层。非预期越界、失效主组件、未绑定solid颜色、顶层重叠均0；22处明确纵向滚动单列。257个Disabled控件无反应，普通文字最低对比度Light **4.53:1** / Dark **6.86:1**。

修复173个局部低对比度文字、12个被HUG压缩的按钮和3个对话框中的其他比赛默认示例。改动只在本页实例/自有节点，没有修改全局组件或变量。

## 历史与实现边界

S01原验收15状态、47条本页/目录连接；S02原验收33状态、7对话框、339条导航/覆盖层连接。原报告中的未接线状态与原计数属于各次验收快照，后续入站增量以最新Screens总状态及当前页记录为准。

完整17路由集成仍未完成。预置输入、任务、失败和结果不是实际数据库、API、文件操作或模型调用。任意选择器、全部P4/P7组合、全状态主题/折叠/草稿/滚动、键盘焦点、后台权限与异步竞态仍需后续实现测试。最终历史隐藏写入没有接到伪造成功结果。

## 换会话恢复

先读本README、`FIGMA_SCREENS_PLAN.md`、`FIGMA_SCREENS_STATE.json`，再读 `FIGMA_SCREEN_PREDICTION_STATE.json` 和本页记录，核对实际Figma根节点。

S02旧总状态保留于 `6aaff66b66fdcdc9f93f2fb74436b9534412f5cc`；S01旧总状态保留于 `7a6891c571ffa07015774ac6b572c93b12c4c315`。各页独立记录继续保留，不将旧截图、AI Chat/Workflow状态或组件记录中的Screens未开始当成当前进度。

`FIGMA_COMPONENT_RECORD.md`的组件API保持不变；不要为了同步阶段状态重写未变化的组件历史。SLOT规范化后的虚拟ID按稳定根、组件引用和语义路径恢复，不猜测节点ID。
