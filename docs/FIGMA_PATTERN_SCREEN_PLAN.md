# Figma Patterns & Screen Plan

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的 Figma 为准。应用源码只用于核对功能入口、交互和数据语义，不再作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

## 当前状态

截至2026-08-28，既定底层组件剩余0组，Controls保持 **61个组件集 /506个变体**。Patterns首组 **PC App Shell** 已完成；Screens（3:5）仍为空。

Shell包含9个组件集 /60个变体、2个生产单组件及1个QA夹具；7个模块 /17个入口的Light、Dark、展开、折叠组合与历史、重试、重置原型已连接并通过结构检查。完整证据见 [独立 Shell 记录](FIGMA_PC_APP_SHELL.md)。

## Patterns 执行顺序

- [x] PC App Shell：一级导航、二级导航、Topbar、折叠与主题保持。
- [ ] **下一组：Page Heading / Toolbar / Filter Bar / Selection Command Bar。**
- [ ] Metric Card / Action Card / Panel。
- [ ] Master–Detail–Inspector 工作区。
- [ ] Workflow Stepper / Timeline。
- [ ] AI Chat：历史侧栏、消息、附件、Composer。

依赖先归档和验收，再用于组合；不在业务页面里新增临时控件。保留各组独立记录，不合并为一份大文档。

## Screens

最后再拼装17条页面链路与7个导航模块。当前Shell中的内容插槽和QA预置数据不算产品Screens。

每个页面必须：

- 只引用已完成的组件和模式实例；
- 覆盖适用的默认、加载、空、错误、禁用、确认与成功反馈；
- 每个按钮、输入、菜单均能追踪组件来源与交互目标；
- 依据Figma确定视觉尺寸，不回到旧代码推导比例。

## 后续实现边界

本阶段仅完成Figma模式与设计记录，没有修改应用代码。原型可展示导航与预置数据变化；真实键盘焦点、网络竞态、表单和滚动持久化需在实现阶段单独测试。
