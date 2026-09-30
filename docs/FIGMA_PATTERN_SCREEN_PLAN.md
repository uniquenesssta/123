# Figma Patterns & Screen Plan

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的 Figma 为准。应用源码只用于核对功能入口、交互和数据语义，不再作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

## 当前状态

截至 **2026-09-30**，既定底层组件剩余0组，Controls保持 **61个组件集 /506个变体**。Patterns 已完成 **PC App Shell**、**Page Heading / Toolbar / Filter Bar / Selection Command Bar**、**Metric Card / Action Card / Panel**、**Master–Detail–Inspector**、**Workflow Stepper / Timeline**；产品 Screens 尚未开始。

Patterns 当前为 **13个组件集 /104个变体 /11个生产单组件**。第五组包含 Workflow Step 的20个 Progress × State variants、Timeline Item 的9个 Verification × Revision variants，以及 Workflow Stepper、Timeline 两个 SLOT 型单组件。新增两个尺寸变量，Foundations 当前为 **193 variables /13 text styles /6 effects**。

Light、Dark、760px 窄宽、空时间线和完整状态矩阵已检查。4个预置原型场景、8条连接及4个禁止写入按钮已读回核对。普通文字最低对比度 Light 4.53:1 / Dark 6.86:1；本组对比度修正不改动既有 Controls 主组件。原型演示不替代实际后端、键盘焦点、权限和异步测试。

完整证据见 [Shell 记录](FIGMA_PC_APP_SHELL.md)、[Page Bars 记录](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card / Panel 记录](FIGMA_METRIC_ACTION_PANEL.md)、[Master–Detail–Inspector 记录](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow Stepper / Timeline 记录](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)。恢复节点与验收结果见 [连续性检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)。

## Patterns 执行顺序

- [x] PC App Shell：一级导航、二级导航、Topbar、折叠与主题保持。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector 工作区（含 Entity Row 依赖）。
- [x] Workflow Stepper / Timeline。
- [ ] **下一组：AI Chat：历史侧栏、消息、附件、Composer。**

依赖先归档和验收，再用于组合；不在业务页面里新增临时控件。保留各组独立记录，不合并为一份大文档。恢复时以最新明确检查点及其对应画布为准，不以旧截图回退阶段。

## Screens

最后再拼装17条页面链路与7个导航模块。当前Shell内容插槽、QA预置数据和 Workflow Lab 原型都不算产品Screens。

每个页面必须：

- 只引用已完成的组件和模式实例；
- 覆盖适用的默认、加载、空、错误、禁用、确认与成功反馈；
- 每个按钮、输入、菜单均能追踪组件来源与交互目标；
- 依据Figma确定视觉尺寸，不回到旧代码推导比例。

## 后续实现边界

本阶段仅完成Figma模式与设计记录，没有修改应用代码。原型可展示导航与预置数据变化；真实键盘焦点、网络竞态、表单和滚动持久化需在实现阶段单独测试。Workflow 的查看步骤与执行权限必须独立；Timeline 的核验、修订、取消记录及有效比分必须由实际业务数据驱动。
