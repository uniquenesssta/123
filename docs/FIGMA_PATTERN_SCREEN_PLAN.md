# Figma Patterns & Screen Plan

本文件只在图标、基础 token 与底层组件验收后执行。

## 当前状态

截至 2026-08-28，既定底层组件组已全部完成并验收，剩余 0 组；Controls 为 **61 个组件集、506 个变体**，最后一组见 [Avatar 记录](FIGMA_AVATAR.md)。

Patterns（`3:4`）和 Screens（`3:5`）仍为空，尚未开始制作。本阶段下一项固定为 **PC App Shell**；先完成模式，再拼装产品页面。发现新组件依赖时，先独立归档和验收，不在页面中临时拼接。

## Patterns

- PC App Shell：一级导航、二级导航、Topbar、折叠状态；
- Page Heading、Toolbar、Filter Bar、Selection Command Bar；
- Metric Card、Action Card、Panel；
- Master–Detail–Inspector 工作区；
- Workflow Stepper / Timeline；
- AI Chat：历史侧栏、消息、附件、Composer。

## Screens

最后再拼装 17 条页面链路与 7 个导航模块。

每个页面必须：

- 只引用已完成的组件实例；
- 覆盖默认、加载、空、错误、禁用、确认、成功反馈等适用状态；
- 每个按钮、输入、菜单都有可追踪组件来源与交互链路；
- 不在页面内新增未归档的临时控件。

## 进入条件

P4.1 图标、基础依赖与既定底层组件均已通过验收，进入条件已满足；本轮仅更新计划，未开始 Patterns 或 Screens。
