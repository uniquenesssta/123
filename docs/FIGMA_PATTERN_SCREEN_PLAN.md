# Figma Patterns & Screen Plan

本文件只在图标、基础 token 与底层组件验收后执行。

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

P4.1 图标、基础依赖与底层组件全部通过验收后，才开始本文件的工作。
