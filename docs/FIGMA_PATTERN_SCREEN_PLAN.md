# Figma Patterns & Screen Plan

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的 Figma 为准。应用源码只用于核对功能入口、交互和数据语义，不再作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

## 当前状态

截至 **2026-09-30**，图标与既定底层组件均完成，Controls保持 **61个组件集 /506个变体**。**Patterns 六组计划全部完成**：PC App Shell、Page Heading / Toolbar / Filter Bar / Selection Command Bar、Metric Card / Action Card / Panel、Master–Detail–Inspector、Workflow Stepper / Timeline、AI Chat。产品 Screens 尚未开始。

Patterns 当前为 **17个组件集 /125个变体 /13个生产单组件**。AI Chat 新增 History Item 10个 Selected × State variants、Attachment 2个只读 Kind variants、Message 3个有效 Kind variants、Composer 6个请求状态，以及 History Sidebar / Workspace 两个 SLOT 型单组件。Foundations 保持 **193 variables /13 text styles /6 effects**。

AI Chat 已检查 Light、Dark、760px 窄宽、完整状态矩阵、上下文显式勾选、空历史、长文本与独立消息滚动。7个预置场景、17条原生连接、A/B 会话隔离及错误/取消草稿保留已读回；深层351个可见嵌套实例无失效引用，非预期越界0。普通文字最低对比度 Light 4.53:1 / Dark 6.86:1。本组没有修改既有 Controls 主组件或应用代码；预置原型不替代运行时异步与键盘测试。

完整证据见 [Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)、[Master–Detail–Inspector](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow / Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。**最新恢复依据：[AI Chat 检查点](FIGMA_AI_CHAT_STATE.json)**；此前 [Workflow 检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json) 作为历史记录保留。

## Patterns 执行顺序

- [x] PC App Shell：一级导航、二级导航、Topbar、折叠与主题保持。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector 工作区（含 Entity Row 依赖）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件、Composer。

依赖已按组归档；保留各组独立记录，不合并为一份大文档。恢复时以最新明确检查点及其对应画布为准，不以旧截图、旧组统计或无内容的 handoff 回退阶段。

## 下一阶段：Screens

- [ ] **产品 Screens：17条页面链路 / 7个导航模块，尚未开始。**

下一次推进先读取页面范围并确定当前屏幕的语义入口、状态与组件依赖，再拼装产品页面；本次不提前创建 Screens。

当前 Shell 内容插槽、QA 预置数据、Workflow Lab、AI Chat Lab 及 AI Chat/Workspace 等可复用 Pattern 都不算已完成的产品 Screens。

每个页面必须：

- 只引用已完成的组件和模式实例；
- 覆盖适用的默认、加载、空、错误、禁用、确认与成功反馈；
- 每个按钮、输入、菜单均能追踪组件来源与交互目标；
- 依据Figma确定视觉尺寸，不回到旧代码推导比例。

## 后续实现边界

本阶段只完成Figma模式与设计记录，没有修改应用代码。原型展示预置数据变化，不替代真实键盘焦点、网络竞态、表单和滚动持久化测试。

Workflow 的查看步骤与执行权限独立；Timeline 的核验、修订、取消及有效比分由真实数据驱动。AI Chat 保持纯文本、只读上下文主动勾选、原会话身份不变；Pending 与取消均绑定 sessionId / requestId，迟到响应不能串入其他会话或覆盖新草稿。附件只覆盖上下文与历史文件记录，不重新加入新上传、生成文件或数据库执行能力。
